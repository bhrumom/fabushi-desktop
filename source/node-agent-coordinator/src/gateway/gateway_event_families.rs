pub const AGENTS: &str = "agents";
pub const TRANSCRIPT: &str = "transcript";
pub const MCP: &str = "mcp";
pub const LIFECYCLE: &str = "lifecycle";

pub fn is_known(family: &str) -> bool {
    matches!(family, AGENTS | TRANSCRIPT | MCP | LIFECYCLE)
}
