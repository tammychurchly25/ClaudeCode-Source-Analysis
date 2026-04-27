// cc-mcp: Model Context Protocol (MCP) client implementation.
//
// MCP is a JSON-RPC 2.0 based protocol for connecting Claude to external
// tool/resource servers. This crate implements:
//
// - JSON-RPC 2.0 client primitives
// - MCP protocol handshake (initialize, initialized)
// - Tool discovery (tools/list)
// - Tool execution (tools/call)
// - Resource management (resources/list, resources/read)
// - Prompt templates (prompts/list, prompts/get)
// - Transport: stdio (subprocess) and HTTP/SSE

use async_trait::async_trait;
use cc_core::config::McpServerConfig;
use cc_core::types::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, warn};

pub use client::McpClient;
pub use types::*;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 Types
// ---------------------------------------------------------------------------

pub mod types {
    use super::*;

    /// A JSON-RPC 2.0 request.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct JsonRpcRequest {
        pub jsonrpc: String,
        pub id: Value,
        pub method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub params: Option<Value>,
    }

    impl JsonRpcRequest {
        pub fn new(id: impl Into<Value>, method: impl Into<String>, params: Option<Value>) -> Self {
            Self {
                jsonrpc: "2.0".to_string(),
                id: id.into(),
                method: method.into(),
                params,
            }
        }

        pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
            Self {
                jsonrpc: "2.0".to_string(),
                id: Value::Null,
                method: method.into(),
                params,
            }
        }
    }

    /// A JSON-RPC 2.0 response.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct JsonRpcResponse {
        pub jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error: Option<JsonRpcError>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct JsonRpcError {
        pub code: i64,
        pub message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<Value>,
    }

    // ---- MCP protocol types ------------------------------------------------

    /// MCP initialize request params.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InitializeParams {
        pub protocol_version: String,
        pub capabilities: ClientCapabilities,
        pub client_info: ClientInfo,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClientCapabilities {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub roots: Option<RootsCapability>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sampling: Option<Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RootsCapability {
        #[serde(rename = "listChanged")]
        pub list_changed: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClientInfo {
        pub name: String,
        pub version: String,
    }

    /// MCP initialize response result.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InitializeResult {
        pub protocol_version: String,
        pub capabilities: ServerCapabilities,
        pub server_info: ServerInfo,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub instructions: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct ServerCapabilities {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tools: Option<ToolsCapability>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub resources: Option<ResourcesCapability>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prompts: Option<PromptsCapability>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub logging: Option<Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ToolsCapability {
        #[serde(default)]
        pub list_changed: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourcesCapability {
        #[serde(default)]
        pub subscribe: bool,
        #[serde(default)]
        pub list_changed: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PromptsCapability {
        #[serde(default)]
        pub list_changed: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ServerInfo {
        pub name: String,
        pub version: String,
    }

    /// An MCP tool definition.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct McpTool {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub input_schema: Value,
    }

    impl From<&McpTool> for ToolDefinition {
        fn from(t: &McpTool) -> Self {
            ToolDefinition {
                name: t.name.clone(),
                description: t.description.clone().unwrap_or_default(),
                input_schema: t.input_schema.clone(),
            }
        }
    }

    /// tools/list response.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ListToolsResult {
        pub tools: Vec<McpTool>,
        #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
        pub next_cursor: Option<String>,
    }

    /// tools/call params.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CallToolParams {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub arguments: Option<Value>,
    }

    /// tools/call response.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CallToolResult {
        pub content: Vec<McpContent>,
        #[serde(default)]
        pub is_error: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum McpContent {
        Text { text: String },
        Image { data: String, #[serde(rename = "mimeType")] mime_type: String },
        Resource { resource: ResourceContents },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResourceContents {
        pub uri: String,
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        pub mime_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub blob: Option<String>,
    }

    /// An MCP resource.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct McpResource {
        pub uri: String,
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mime_type: Option<String>,
    }

    /// resources/list response.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ListResourcesResult {
        pub resources: Vec<McpResource>,
        #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
        pub next_cursor: Option<String>,
    }

    /// An MCP prompt template.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpPrompt {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(default)]
        pub arguments: Vec<McpPromptArgument>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpPromptArgument {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(default)]
        pub required: bool,
    }

    /// prompts/list response.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ListPromptsResult {
        pub prompts: Vec<McpPrompt>,
    }
}

// ---------------------------------------------------------------------------
// Transport layer
// ---------------------------------------------------------------------------

pub mod transport {
    use super::*;

    /// A transport can send requests and receive responses.
    #[async_trait]
    pub trait McpTransport: Send + Sync {
        async fn send(&self, message: &JsonRpcRequest) -> anyhow::Result<()>;
        async fn recv(&self) -> anyhow::Result<Option<JsonRpcResponse>>;
        async fn close(&self) -> anyhow::Result<()>;
    }

    /// Stdio transport: spawns a subprocess and communicates via stdin/stdout.
    pub struct StdioTransport {
        child: Arc<Mutex<Child>>,
        stdin: Arc<Mutex<ChildStdin>>,
        stdout_rx: Arc<Mutex<mpsc::UnboundedReceiver<String>>>,
    }

    impl StdioTransport {
        pub async fn spawn(config: &McpServerConfig) -> anyhow::Result<Self> {
            let command = config
                .command
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("MCP server '{}' has no command", config.name))?;

            let mut cmd = Command::new(command);
            cmd.args(&config.args)
                .envs(&config.env)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn()?;

            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| anyhow::anyhow!("Could not get stdin"))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("Could not get stdout"))?;

            let (tx, rx) = mpsc::unbounded_channel::<String>();

            // Background reader task
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx.send(line).is_err() {
                        break;
                    }
                }
            });

            Ok(Self {
                child: Arc::new(Mutex::new(child)),
                stdin: Arc::new(Mutex::new(stdin)),
                stdout_rx: Arc::new(Mutex::new(rx)),
            })
        }
    }

    #[async_trait]
    impl McpTransport for StdioTransport {
        async fn send(&self, message: &JsonRpcRequest) -> anyhow::Result<()> {
            let json = serde_json::to_string(message)? + "\n";
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(json.as_bytes()).await?;
            stdin.flush().await?;
            Ok(())
        }

        async fn recv(&self) -> anyhow::Result<Option<JsonRpcResponse>> {
            let mut rx = self.stdout_rx.lock().await;
            let line = rx.recv().await;
            match line {
                Some(s) => {
                    let resp: JsonRpcResponse = serde_json::from_str(&s)?;
                    Ok(Some(resp))
                }
                None => Ok(None),
            }
        }

        async fn close(&self) -> anyhow::Result<()> {
            let mut child = self.child.lock().await;
            let _ = child.kill().await;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// MCP Client
// ---------------------------------------------------------------------------

pub mod client {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A fully initialized MCP client connected to a single server.
    pub struct McpClient {
        pub server_name: String,
        pub server_info: Option<ServerInfo>,
        pub capabilities: ServerCapabilities,
        pub tools: Vec<McpTool>,
        pub resources: Vec<McpResource>,
        pub prompts: Vec<McpPrompt>,
        transport: Arc<dyn transport::McpTransport>,
        next_id: AtomicU64,
        pending: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
    }

    impl McpClient {
        /// Connect to an MCP server using stdio transport and complete the
        /// initialize handshake.
        pub async fn connect_stdio(config: &McpServerConfig) -> anyhow::Result<Self> {
            let transport = transport::StdioTransport::spawn(config).await?;
            let client = Self {
                server_name: config.name.clone(),
                server_info: None,
                capabilities: ServerCapabilities::default(),
                tools: vec![],
                resources: vec![],
                prompts: vec![],
                transport: Arc::new(transport),
                next_id: AtomicU64::new(1),
                pending: Arc::new(Mutex::new(HashMap::new())),
            };

            client.initialize().await
        }

        /// Send the MCP initialize handshake and discover capabilities.
        async fn initialize(mut self) -> anyhow::Result<Self> {
            let params = InitializeParams {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ClientCapabilities {
                    roots: Some(RootsCapability { list_changed: false }),
                    sampling: None,
                },
                client_info: ClientInfo {
                    name: cc_core::constants::APP_NAME.to_string(),
                    version: cc_core::constants::APP_VERSION.to_string(),
                },
            };

            let result: InitializeResult = self
                .call("initialize", Some(serde_json::to_value(&params)?))
                .await?;

            self.server_info = Some(result.server_info);
            self.capabilities = result.capabilities.clone();

            // Send initialized notification
            let notif = JsonRpcRequest::notification("notifications/initialized", None);
            self.transport.send(&notif).await?;

            // Discover tools if supported
            if result.capabilities.tools.is_some() {
                match self.list_tools().await {
                    Ok(tools) => self.tools = tools,
                    Err(e) => warn!(server = %self.server_name, error = %e, "Failed to list tools"),
                }
            }

            // Discover resources if supported
            if result.capabilities.resources.is_some() {
                match self.list_resources().await {
                    Ok(resources) => self.resources = resources,
                    Err(e) => warn!(server = %self.server_name, error = %e, "Failed to list resources"),
                }
            }

            // Discover prompts if supported
            if result.capabilities.prompts.is_some() {
                match self.list_prompts().await {
                    Ok(prompts) => self.prompts = prompts,
                    Err(e) => warn!(server = %self.server_name, error = %e, "Failed to list prompts"),
                }
            }

            Ok(self)
        }

        // ---- High-level API -----------------------------------------------

        pub async fn list_tools(&self) -> anyhow::Result<Vec<McpTool>> {
            let result: ListToolsResult = self.call("tools/list", None).await?;
            Ok(result.tools)
        }

        pub async fn call_tool(
            &self,
            name: &str,
            arguments: Option<Value>,
        ) -> anyhow::Result<CallToolResult> {
            let params = CallToolParams {
                name: name.to_string(),
                arguments,
            };
            self.call("tools/call", Some(serde_json::to_value(&params)?))
                .await
        }

        pub async fn list_resources(&self) -> anyhow::Result<Vec<McpResource>> {
            let result: ListResourcesResult = self.call("resources/list", None).await?;
            Ok(result.resources)
        }

        pub async fn read_resource(&self, uri: &str) -> anyhow::Result<ResourceContents> {
            let params = serde_json::json!({ "uri": uri });
            let result: Value = self.call("resources/read", Some(params)).await?;
            let contents = result
                .get("contents")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .ok_or_else(|| anyhow::anyhow!("No contents in response"))?;
            Ok(serde_json::from_value(contents.clone())?)
        }

        pub async fn list_prompts(&self) -> anyhow::Result<Vec<McpPrompt>> {
            let result: ListPromptsResult = self.call("prompts/list", None).await?;
            Ok(result.prompts)
        }

        /// Get all tools as `ToolDefinition` objects suitable for the API.
        pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
            self.tools.iter().map(|t| t.into()).collect()
        }

        // ---- Internal RPC machinery ---------------------------------------

        /// Send a request and wait for the response, deserializing into T.
        async fn call<T: for<'de> Deserialize<'de>>(
            &self,
            method: &str,
            params: Option<Value>,
        ) -> anyhow::Result<T> {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let req = JsonRpcRequest::new(id, method, params);

            // We use a simple request/response loop here (no concurrent requests).
            // For production use, proper demultiplexing by id would be needed.
            self.transport.send(&req).await?;

            loop {
                let resp = self
                    .transport
                    .recv()
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("MCP transport closed"))?;

                // Check if this response matches our request id
                let resp_id = resp.id.as_ref().and_then(|v| v.as_u64()).unwrap_or(0);
                if resp_id != id {
                    // Might be a server-initiated notification; skip
                    debug!(got_id = resp_id, want_id = id, "Skipping non-matching response");
                    continue;
                }

                if let Some(err) = resp.error {
                    return Err(anyhow::anyhow!(
                        "MCP error {}: {}",
                        err.code,
                        err.message
                    ));
                }

                let result = resp
                    .result
                    .ok_or_else(|| anyhow::anyhow!("No result in MCP response"))?;
                return Ok(serde_json::from_value(result)?);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MCP Manager: manages multiple server connections
// ---------------------------------------------------------------------------

/// Manages a pool of MCP server connections.
pub struct McpManager {
    clients: HashMap<String, McpClient>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Connect to all configured MCP servers.
    pub async fn connect_all(configs: &[McpServerConfig]) -> Self {
        let mut manager = Self::new();
        for config in configs {
            match config.server_type.as_str() {
                "stdio" => {
                    debug!(server = %config.name, "Connecting to MCP server via stdio");
                    match McpClient::connect_stdio(config).await {
                        Ok(client) => {
                            let name = config.name.clone();
                            manager.clients.insert(name, client);
                        }
                        Err(e) => {
                            error!(
                                server = %config.name,
                                error = %e,
                                "Failed to connect to MCP server"
                            );
                        }
                    }
                }
                other => {
                    warn!(transport = other, "Unsupported MCP transport type");
                }
            }
        }
        manager
    }

    /// Get all tool definitions from all connected servers.
    pub fn all_tool_definitions(&self) -> Vec<(String, ToolDefinition)> {
        let mut defs = vec![];
        for (server_name, client) in &self.clients {
            for td in client.tool_definitions() {
                // Prefix tool name with server name to avoid conflicts
                let prefixed = ToolDefinition {
                    name: format!("{}_{}", server_name, td.name),
                    description: format!("[{}] {}", server_name, td.description),
                    input_schema: td.input_schema.clone(),
                };
                defs.push((server_name.clone(), prefixed));
            }
        }
        defs
    }

    /// Execute a tool call, routing to the correct server.
    /// Tool name format: `<server_name>_<tool_name>`.
    pub async fn call_tool(
        &self,
        prefixed_name: &str,
        arguments: Option<Value>,
    ) -> anyhow::Result<CallToolResult> {
        // Find the server name by matching prefix
        for (server_name, client) in &self.clients {
            let prefix = format!("{}_", server_name);
            if let Some(tool_name) = prefixed_name.strip_prefix(&prefix) {
                return client.call_tool(tool_name, arguments).await;
            }
        }
        Err(anyhow::anyhow!(
            "No MCP server found for tool: {}",
            prefixed_name
        ))
    }

    /// Number of connected servers.
    pub fn server_count(&self) -> usize {
        self.clients.len()
    }

    /// List all connected server names.
    pub fn server_names(&self) -> Vec<&str> {
        self.clients.keys().map(|s| s.as_str()).collect()
    }

    /// Get server instructions (from initialize response).
    pub fn server_instructions(&self) -> Vec<(String, String)> {
        // McpClient doesn't store instructions yet; placeholder
        vec![]
    }

    /// List all resources from all (or a specific) connected server.
    pub async fn list_all_resources(
        &self,
        server_filter: Option<&str>,
    ) -> Vec<serde_json::Value> {
        let mut all = vec![];
        for (name, client) in &self.clients {
            if let Some(filter) = server_filter {
                if name != filter {
                    continue;
                }
            }
            match client.list_resources().await {
                Ok(resources) => {
                    for r in resources {
                        all.push(serde_json::json!({
                            "uri": r.uri,
                            "name": r.name,
                            "description": r.description,
                            "mimeType": r.mime_type,
                            "server": name,
                        }));
                    }
                }
                Err(e) => {
                    warn!(server = %name, error = %e, "Failed to list resources");
                }
            }
        }
        all
    }

    /// Read a specific resource from a named server.
    pub async fn read_resource(
        &self,
        server_name: &str,
        uri: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let client = self
            .clients
            .get(server_name)
            .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", server_name))?;

        let contents = client.read_resource(uri).await?;
        Ok(serde_json::to_value(&contents)?)
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MCP Tool wrapper: makes MCP tools act like native cc-tools
// ---------------------------------------------------------------------------
// (This would be in cc-tools but is here to avoid circular deps)

/// Convert MCP tool call result to a string for the model.
pub fn mcp_result_to_string(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| match c {
            McpContent::Text { text } => Some(text.as_str()),
            McpContent::Image { .. } => Some("[image]"),
            McpContent::Resource { resource } => {
                resource.text.as_deref().or(Some("[binary resource]"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serialization() {
        let req = JsonRpcRequest::new(1u64, "tools/list", None);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn test_mcp_tool_to_definition() {
        let tool = McpTool {
            name: "search".to_string(),
            description: Some("Search the web".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "query": { "type": "string" } }
            }),
        };
        let def: ToolDefinition = (&tool).into();
        assert_eq!(def.name, "search");
        assert_eq!(def.description, "Search the web");
    }
}
