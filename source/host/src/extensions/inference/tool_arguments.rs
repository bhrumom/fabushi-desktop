use serde_json::Value;

/// Provider syntax errors are tool feedback, never invented default arguments.
pub(super) fn parse_tool_arguments(raw: &str) -> Result<Value, String> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        format!("Invalid tool arguments: expected a JSON object (line {}, column {}). No tool was executed.", error.line(), error.column())
    })?;
    if !value.is_object() {
        return Err("Invalid tool arguments: expected a JSON object. No tool was executed.".into());
    }
    Ok(value)
}
