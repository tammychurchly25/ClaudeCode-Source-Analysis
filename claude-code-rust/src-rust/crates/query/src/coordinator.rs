//! Coordinator mode: multi-worker agent orchestration

use crate::*;

pub const COORDINATOR_ENV_VAR: &str = "CLAUDE_CODE_COORDINATOR_MODE";

pub fn is_coordinator_mode() -> bool {
    std::env::var(COORDINATOR_ENV_VAR)
        .map(|v| !v.is_empty() && v != "0" && v != "false")
        .unwrap_or(false)
}

/// System prompt sections injected when coordinator mode is active
pub fn coordinator_system_prompt() -> &'static str {
    r#"
## Coordinator Mode

You are operating as an orchestrator for parallel worker agents.

### Your Role
- Orchestrate workers using the Agent tool to spawn parallel subagents
- Use SendMessage to continue communication with running workers
- Use TaskStop to cancel workers that are no longer needed
- Synthesize findings across workers before presenting to the user
- Answer directly when the question doesn't need delegation

### Task Workflow
1. **Research Phase**: Spawn workers to gather information in parallel
2. **Synthesis Phase**: Collect and merge worker findings
3. **Implementation Phase**: Delegate implementation tasks to specialized workers
4. **Verification Phase**: Spawn verification workers to validate results

### Worker Guidelines
- Worker prompts must be fully self-contained (workers cannot see your conversation)
- Always synthesize findings before spawning follow-up workers
- Workers have access to all standard tools + MCP + skills
- Use TaskCreate/TaskUpdate to track parallel work

### Internal Tools (do not delegate to workers)
- Agent, SendMessage, TaskStop (coordination only)
"#
}

/// Tools that should NOT be passed to worker agents
pub const INTERNAL_COORDINATOR_TOOLS: &[&str] = &[
    "Agent",
    "SendMessage",
    "TaskStop",
];

/// Get the user context injected for coordinator sessions
pub fn coordinator_user_context(available_tools: &[String], mcp_servers: &[String]) -> String {
    let tool_list = available_tools
        .iter()
        .filter(|t| !INTERNAL_COORDINATOR_TOOLS.contains(&t.as_str()))
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    let mcp_section = if mcp_servers.is_empty() {
        String::new()
    } else {
        format!("\nConnected MCP servers: {}", mcp_servers.join(", "))
    };

    format!(
        "Available worker tools: {}{}\n",
        tool_list, mcp_section
    )
}

/// Check if session mode matches current coordinator setting, returns warning if mismatched
pub fn match_session_mode(stored_coordinator: bool) -> Option<String> {
    let current = is_coordinator_mode();
    if stored_coordinator != current {
        if current {
            std::env::set_var(COORDINATOR_ENV_VAR, "1");
        } else {
            std::env::remove_var(COORDINATOR_ENV_VAR);
        }
        Some(format!(
            "Session was created in {} mode, switching to match.",
            if stored_coordinator { "coordinator" } else { "standard" }
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_coordinator_mode_unset() {
        std::env::remove_var(COORDINATOR_ENV_VAR);
        assert!(!is_coordinator_mode());
    }

    #[test]
    fn test_is_coordinator_mode_set_to_one() {
        std::env::set_var(COORDINATOR_ENV_VAR, "1");
        assert!(is_coordinator_mode());
        std::env::remove_var(COORDINATOR_ENV_VAR);
    }

    #[test]
    fn test_is_coordinator_mode_set_to_false() {
        std::env::set_var(COORDINATOR_ENV_VAR, "false");
        assert!(!is_coordinator_mode());
        std::env::remove_var(COORDINATOR_ENV_VAR);
    }

    #[test]
    fn test_is_coordinator_mode_set_to_zero() {
        std::env::set_var(COORDINATOR_ENV_VAR, "0");
        assert!(!is_coordinator_mode());
        std::env::remove_var(COORDINATOR_ENV_VAR);
    }

    #[test]
    fn test_coordinator_user_context_filters_internal_tools() {
        let tools = vec![
            "Bash".to_string(),
            "Agent".to_string(),
            "SendMessage".to_string(),
            "TaskStop".to_string(),
            "Read".to_string(),
        ];
        let ctx = coordinator_user_context(&tools, &[]);
        assert!(ctx.contains("Bash"));
        assert!(ctx.contains("Read"));
        assert!(!ctx.contains("Agent"));
        assert!(!ctx.contains("SendMessage"));
        assert!(!ctx.contains("TaskStop"));
    }

    #[test]
    fn test_coordinator_user_context_mcp_servers() {
        let tools = vec!["Bash".to_string()];
        let mcps = vec!["filesystem".to_string(), "git".to_string()];
        let ctx = coordinator_user_context(&tools, &mcps);
        assert!(ctx.contains("filesystem"));
        assert!(ctx.contains("git"));
    }

    #[test]
    fn test_match_session_mode_no_change_needed() {
        std::env::remove_var(COORDINATOR_ENV_VAR);
        // current = false, stored = false → no warning
        assert!(match_session_mode(false).is_none());
    }

    #[test]
    fn test_match_session_mode_switches_to_coordinator() {
        std::env::remove_var(COORDINATOR_ENV_VAR);
        // current = false, stored = true → should flip and warn
        let msg = match_session_mode(true);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("coordinator"));
        // Clean up
        std::env::remove_var(COORDINATOR_ENV_VAR);
    }

    #[test]
    fn test_coordinator_system_prompt_content() {
        let prompt = coordinator_system_prompt();
        assert!(prompt.contains("Coordinator Mode"));
        assert!(prompt.contains("orchestrator"));
        assert!(prompt.contains("Research Phase"));
        assert!(prompt.contains("Synthesis Phase"));
    }
}
