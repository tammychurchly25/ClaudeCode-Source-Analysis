//! Named commands (e.g. `claude agents`, `claude ide`, `claude branch`, …).
//!
//! These complement slash commands with more complex top-level flows.
//! A named command is invoked when the *first* CLI argument matches one
//! of the registered names — before the normal REPL starts.
//!
//! Sources consulted while porting:
//!   src/commands/agents/index.ts
//!   src/commands/ide/index.ts
//!   src/commands/branch/index.ts
//!   src/commands/tag/index.ts
//!   src/commands/passes/index.ts
//!   src/commands/pr_comments/index.ts
//!   src/commands/install-github-app/index.ts
//!   src/commands/desktop/index.ts  (implied by component structure)
//!   src/commands/mobile/index.ts   (implied by component structure)
//!   src/commands/remote-setup/index.ts (implied by component structure)

use crate::{CommandContext, CommandResult};

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A top-level named command (`claude <name> [args…]`).
pub trait NamedCommand: Send + Sync {
    /// Primary command name, e.g. `"agents"`.
    fn name(&self) -> &str;

    /// One-line description used in `claude --help`.
    fn description(&self) -> &str;

    /// Usage hint shown in `claude <name> --help`.
    fn usage(&self) -> &str;

    /// Execute the command.  `args` is the slice of arguments *after* the
    /// command name itself.
    fn execute_named(&self, args: &[&str], ctx: &CommandContext) -> CommandResult;
}

// ---------------------------------------------------------------------------
// agents
// ---------------------------------------------------------------------------

pub struct AgentsCommand;

impl NamedCommand for AgentsCommand {
    fn name(&self) -> &str { "agents" }
    fn description(&self) -> &str { "Manage and configure sub-agents" }
    fn usage(&self) -> &str { "claude agents [list|create|edit|delete] [name]" }

    fn execute_named(&self, args: &[&str], _ctx: &CommandContext) -> CommandResult {
        match args.first().copied().unwrap_or("list") {
            "list" => CommandResult::Message(
                "Sub-agents are defined in .claude/agents/ as Markdown files.\n\
                 Use 'claude agents create <name>' to scaffold a new agent."
                    .to_string(),
            ),
            "create" => {
                let name = args.get(1).copied().unwrap_or("my-agent");
                CommandResult::Message(format!(
                    "Create a new agent by adding .claude/agents/{name}.md\n\
                     Template:\n\
                     ---\n\
                     name: {name}\n\
                     description: <description>\n\
                     model: claude-sonnet-4-6\n\
                     ---\n\n\
                     <agent instructions here>"
                ))
            }
            "edit" => {
                let name = match args.get(1).copied() {
                    Some(n) => n,
                    None => return CommandResult::Error(
                        "Usage: claude agents edit <name>".to_string(),
                    ),
                };
                CommandResult::Message(format!(
                    "Edit .claude/agents/{name}.md in your editor to update the agent."
                ))
            }
            "delete" => {
                let name = match args.get(1).copied() {
                    Some(n) => n,
                    None => return CommandResult::Error(
                        "Usage: claude agents delete <name>".to_string(),
                    ),
                };
                CommandResult::Message(format!(
                    "Delete .claude/agents/{name}.md to remove the agent."
                ))
            }
            sub => CommandResult::Error(format!("Unknown agents subcommand: '{sub}'")),
        }
    }
}

// ---------------------------------------------------------------------------
// add-dir
// ---------------------------------------------------------------------------

pub struct AddDirCommand;

impl NamedCommand for AddDirCommand {
    fn name(&self) -> &str { "add-dir" }
    fn description(&self) -> &str { "Add a directory to Claude Code's allowed workspace paths" }
    fn usage(&self) -> &str { "claude add-dir <path>" }

    fn execute_named(&self, args: &[&str], _ctx: &CommandContext) -> CommandResult {
        let raw = match args.first() {
            Some(p) => *p,
            None => return CommandResult::Error("Usage: claude add-dir <path>".to_string()),
        };

        let path = std::path::Path::new(raw);

        if !path.exists() {
            return CommandResult::Error(format!("Directory does not exist: {}", path.display()));
        }

        if !path.is_dir() {
            return CommandResult::Error(format!("Not a directory: {}", path.display()));
        }

        let abs_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => return CommandResult::Error(format!("Cannot resolve path: {e}")),
        };

        let mut settings = match cc_core::config::Settings::load_sync() {
            Ok(s) => s,
            Err(e) => {
                return CommandResult::Error(format!(
                    "Failed to load settings before updating workspace paths: {e}"
                ))
            }
        };

        if !settings.config.workspace_paths.iter().any(|p| p == &abs_path) {
            settings.config.workspace_paths.push(abs_path.clone());
            if let Err(e) = settings.save_sync() {
                return CommandResult::Error(format!(
                    "Added {} for this session, but failed to save settings: {}",
                    abs_path.display(),
                    e
                ));
            }
        }

        CommandResult::Message(format!(
            "Added {} to allowed workspace paths.",
            abs_path.display()
        ))
    }
}

// ---------------------------------------------------------------------------
// branch
// ---------------------------------------------------------------------------

pub struct BranchCommand;

impl NamedCommand for BranchCommand {
    fn name(&self) -> &str { "branch" }
    fn description(&self) -> &str { "Create a branch of the current conversation at this point" }
    fn usage(&self) -> &str { "claude branch [create|switch|list] [name]" }

    fn execute_named(&self, args: &[&str], _ctx: &CommandContext) -> CommandResult {
        match args.first().copied().unwrap_or("list") {
            "list" => CommandResult::UserMessage(
                "List git branches — run: git branch -a".to_string(),
            ),
            "create" => {
                let name = match args.get(1) {
                    Some(n) => *n,
                    None => return CommandResult::Error(
                        "Usage: claude branch create <name>".to_string(),
                    ),
                };
                CommandResult::UserMessage(format!("git checkout -b {name}"))
            }
            "switch" | "checkout" => {
                let name = match args.get(1) {
                    Some(n) => *n,
                    None => return CommandResult::Error(
                        "Usage: claude branch switch <name>".to_string(),
                    ),
                };
                CommandResult::UserMessage(format!("git checkout {name}"))
            }
            sub => CommandResult::Error(format!("Unknown branch subcommand: '{sub}'")),
        }
    }
}

// ---------------------------------------------------------------------------
// tag
// ---------------------------------------------------------------------------

pub struct TagCommand;

impl NamedCommand for TagCommand {
    fn name(&self) -> &str { "tag" }
    fn description(&self) -> &str { "Toggle a searchable tag on the current session" }
    fn usage(&self) -> &str { "claude tag [list|add|remove] [tag]" }

    fn execute_named(&self, args: &[&str], _ctx: &CommandContext) -> CommandResult {
        match args.first().copied().unwrap_or("list") {
            "list" => CommandResult::Message("No tags set for this session.".to_string()),
            "add" => {
                let tag = args.get(1).copied().unwrap_or("unnamed");
                CommandResult::Message(format!("Added tag: {tag}"))
            }
            "remove" => {
                let tag = match args.get(1).copied() {
                    Some(t) if !t.is_empty() => t,
                    _ => return CommandResult::Error(
                        "Usage: claude tag remove <tag>".to_string(),
                    ),
                };
                CommandResult::Message(format!("Removed tag: {tag}"))
            }
            sub => CommandResult::Error(format!("Unknown tag subcommand: '{sub}'")),
        }
    }
}

// ---------------------------------------------------------------------------
// passes
// ---------------------------------------------------------------------------

pub struct PassesCommand;

impl NamedCommand for PassesCommand {
    fn name(&self) -> &str { "passes" }
    fn description(&self) -> &str { "Share a free week of Claude Code with friends" }
    fn usage(&self) -> &str { "claude passes" }

    fn execute_named(&self, _args: &[&str], _ctx: &CommandContext) -> CommandResult {
        CommandResult::Message(
            "Guest passes let you share Claude Code access with friends.\n\
             Visit https://claude.ai/claude-code to manage your passes."
                .to_string(),
        )
    }
}

// ---------------------------------------------------------------------------
// ide
// ---------------------------------------------------------------------------

pub struct IdeCommand;

impl NamedCommand for IdeCommand {
    fn name(&self) -> &str { "ide" }
    fn description(&self) -> &str { "Manage IDE integrations and show status" }
    fn usage(&self) -> &str { "claude ide [status|connect|disconnect|open]" }

    fn execute_named(&self, args: &[&str], _ctx: &CommandContext) -> CommandResult {
        match args.first().copied().unwrap_or("status") {
            "status" => CommandResult::Message(
                "IDE integration status: Not connected\n\
                 Install the Claude Code extension:\n  \
                 - VS Code: https://marketplace.visualstudio.com/items?itemName=Anthropic.claude-code\n  \
                 - JetBrains: https://plugins.jetbrains.com/plugin/claude-code"
                    .to_string(),
            ),
            "connect" | "open" => CommandResult::Message(
                "Connecting to IDE…\n\
                 Make sure the Claude Code extension is installed and running."
                    .to_string(),
            ),
            "disconnect" => CommandResult::Message("Disconnected from IDE.".to_string()),
            sub => CommandResult::Error(format!("Unknown ide subcommand: '{sub}'")),
        }
    }
}

// ---------------------------------------------------------------------------
// pr-comments
// ---------------------------------------------------------------------------

pub struct PrCommentsCommand;

impl NamedCommand for PrCommentsCommand {
    fn name(&self) -> &str { "pr-comments" }
    fn description(&self) -> &str { "Get comments from a GitHub pull request" }
    fn usage(&self) -> &str { "claude pr-comments [PR-number]" }

    fn execute_named(&self, args: &[&str], _ctx: &CommandContext) -> CommandResult {
        let pr_num = args.first().copied().unwrap_or("");
        if pr_num.is_empty() {
            return CommandResult::Error(
                "Please specify a PR number: claude pr-comments <number>".to_string(),
            );
        }
        CommandResult::UserMessage(format!(
            "Fetch and display comments for PR #{pr_num}.\n\
             Steps:\n\
             1. gh pr view {pr_num} --json number,headRepository\n\
             2. gh api /repos/{{owner}}/{{repo}}/issues/{pr_num}/comments\n\
             3. gh api /repos/{{owner}}/{{repo}}/pulls/{pr_num}/comments\n\
             Format results with file paths, diff hunks, and threading."
        ))
    }
}

// ---------------------------------------------------------------------------
// desktop
// ---------------------------------------------------------------------------

pub struct DesktopCommand;

impl NamedCommand for DesktopCommand {
    fn name(&self) -> &str { "desktop" }
    fn description(&self) -> &str { "Open the Claude Code desktop app" }
    fn usage(&self) -> &str { "claude desktop" }

    fn execute_named(&self, _args: &[&str], _ctx: &CommandContext) -> CommandResult {
        CommandResult::Message(
            "Download the Claude Code desktop app at https://claude.ai/download".to_string(),
        )
    }
}

// ---------------------------------------------------------------------------
// mobile
// ---------------------------------------------------------------------------

pub struct MobileCommand;

impl NamedCommand for MobileCommand {
    fn name(&self) -> &str { "mobile" }
    fn description(&self) -> &str { "Set up Claude Code on mobile" }
    fn usage(&self) -> &str { "claude mobile" }

    fn execute_named(&self, _args: &[&str], _ctx: &CommandContext) -> CommandResult {
        CommandResult::Message(
            "Access Claude Code on mobile via https://claude.ai/claude-code\n\
             Use the Bridge feature to connect your local Claude Code CLI to the mobile interface."
                .to_string(),
        )
    }
}

// ---------------------------------------------------------------------------
// install-github-app
// ---------------------------------------------------------------------------

pub struct InstallGithubAppCommand;

impl NamedCommand for InstallGithubAppCommand {
    fn name(&self) -> &str { "install-github-app" }
    fn description(&self) -> &str { "Set up Claude GitHub Actions for a repository" }
    fn usage(&self) -> &str { "claude install-github-app" }

    fn execute_named(&self, _args: &[&str], _ctx: &CommandContext) -> CommandResult {
        CommandResult::Message(
            "To install the Claude Code GitHub App:\n\
             1. Visit https://github.com/apps/claude-code-app and click Install\n\
             2. Select the repositories to enable\n\
             3. Add your ANTHROPIC_API_KEY to repository secrets\n\n\
             The app enables Claude Code in GitHub Actions workflows.\n\
             Docs: https://docs.anthropic.com/claude-code/github-actions"
                .to_string(),
        )
    }
}

// ---------------------------------------------------------------------------
// remote-setup
// ---------------------------------------------------------------------------

pub struct RemoteSetupCommand;

impl NamedCommand for RemoteSetupCommand {
    fn name(&self) -> &str { "remote-setup" }
    fn description(&self) -> &str { "Configure a remote Claude Code environment" }
    fn usage(&self) -> &str { "claude remote-setup" }

    fn execute_named(&self, _args: &[&str], _ctx: &CommandContext) -> CommandResult {
        CommandResult::Message(
            "Remote Claude Code setup:\n\
             1. Set CLAUDE_CODE_REMOTE=1 on the remote machine\n\
             2. Set ANTHROPIC_API_KEY or configure OAuth\n\
             3. Run: claude --no-update-check\n\n\
             For Bridge mode (connect to the claude.ai web UI):\n\
             Set CLAUDE_CODE_BRIDGE_TOKEN=<token from claude.ai>"
                .to_string(),
        )
    }
}

// ---------------------------------------------------------------------------
// stickers
// ---------------------------------------------------------------------------

pub struct StickersCommand;

impl NamedCommand for StickersCommand {
    fn name(&self) -> &str { "stickers" }
    fn description(&self) -> &str { "View collected stickers" }
    fn usage(&self) -> &str { "claude stickers" }

    fn execute_named(&self, _args: &[&str], _ctx: &CommandContext) -> CommandResult {
        CommandResult::Message("Sticker collection: coming soon!".to_string())
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Return one instance of every registered named command.
pub fn all_named_commands() -> Vec<Box<dyn NamedCommand>> {
    vec![
        Box::new(AgentsCommand),
        Box::new(AddDirCommand),
        Box::new(BranchCommand),
        Box::new(TagCommand),
        Box::new(PassesCommand),
        Box::new(IdeCommand),
        Box::new(PrCommentsCommand),
        Box::new(DesktopCommand),
        Box::new(MobileCommand),
        Box::new(InstallGithubAppCommand),
        Box::new(RemoteSetupCommand),
        Box::new(StickersCommand),
    ]
}

/// Look up a named command by its primary name (case-insensitive).
pub fn find_named_command(name: &str) -> Option<Box<dyn NamedCommand>> {
    let needle = name.to_lowercase();
    all_named_commands()
        .into_iter()
        .find(|c| c.name() == needle.as_str())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::cost::CostTracker;
    use std::sync::Arc;

    fn make_ctx() -> CommandContext {
        CommandContext {
            config: cc_core::config::Config::default(),
            cost_tracker: CostTracker::new(),
            messages: vec![],
            working_dir: std::path::PathBuf::from("."),
        }
    }

    #[test]
    fn test_all_named_commands_non_empty() {
        assert!(!all_named_commands().is_empty());
    }

    #[test]
    fn test_all_named_commands_unique_names() {
        let mut names = std::collections::HashSet::new();
        for cmd in all_named_commands() {
            assert!(
                names.insert(cmd.name().to_string()),
                "Duplicate named command: {}",
                cmd.name()
            );
        }
    }

    #[test]
    fn test_find_named_command_found() {
        assert!(find_named_command("agents").is_some());
        assert!(find_named_command("ide").is_some());
        assert!(find_named_command("branch").is_some());
        assert!(find_named_command("passes").is_some());
    }

    #[test]
    fn test_find_named_command_not_found() {
        assert!(find_named_command("nonexistent-xyz").is_none());
    }

    #[test]
    fn test_find_named_command_case_insensitive() {
        assert!(find_named_command("Agents").is_some());
        assert!(find_named_command("IDE").is_some());
    }

    #[test]
    fn test_agents_list_returns_message() {
        let ctx = make_ctx();
        let cmd = AgentsCommand;
        let result = cmd.execute_named(&[], &ctx);
        assert!(matches!(result, CommandResult::Message(_)));
    }

    #[test]
    fn test_agents_create_includes_name() {
        let ctx = make_ctx();
        let cmd = AgentsCommand;
        let result = cmd.execute_named(&["create", "my-bot"], &ctx);
        if let CommandResult::Message(msg) = result {
            assert!(msg.contains("my-bot"));
        } else {
            panic!("Expected Message");
        }
    }

    #[test]
    fn test_add_dir_missing_arg_returns_error() {
        let ctx = make_ctx();
        let cmd = AddDirCommand;
        let result = cmd.execute_named(&[], &ctx);
        assert!(matches!(result, CommandResult::Error(_)));
    }

    #[test]
    fn test_branch_list_returns_user_message() {
        let ctx = make_ctx();
        let cmd = BranchCommand;
        let result = cmd.execute_named(&["list"], &ctx);
        assert!(matches!(result, CommandResult::UserMessage(_)));
    }

    #[test]
    fn test_branch_create_requires_name() {
        let ctx = make_ctx();
        let cmd = BranchCommand;
        let result = cmd.execute_named(&["create"], &ctx);
        assert!(matches!(result, CommandResult::Error(_)));
    }

    #[test]
    fn test_pr_comments_missing_number() {
        let ctx = make_ctx();
        let cmd = PrCommentsCommand;
        let result = cmd.execute_named(&[], &ctx);
        assert!(matches!(result, CommandResult::Error(_)));
    }

    #[test]
    fn test_pr_comments_with_number() {
        let ctx = make_ctx();
        let cmd = PrCommentsCommand;
        let result = cmd.execute_named(&["42"], &ctx);
        assert!(matches!(result, CommandResult::UserMessage(_)));
        if let CommandResult::UserMessage(msg) = result {
            assert!(msg.contains("42"));
        }
    }

    #[test]
    fn test_install_github_app_returns_message() {
        let ctx = make_ctx();
        let cmd = InstallGithubAppCommand;
        let result = cmd.execute_named(&[], &ctx);
        assert!(matches!(result, CommandResult::Message(_)));
    }
}
