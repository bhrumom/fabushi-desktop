#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolProjectionPhase {
    Started,
    Partial,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectedClientSideToolV2 {
    Call {
        tool_call_id: String,
        name: String,
        raw_args: String,
        model_call_id: Option<String>,
        is_streaming: bool,
    },
    Result {
        tool_call_id: String,
        name: String,
        output: String,
        error: Option<String>,
    },
}

pub fn project_basic_tool_call(
    phase: ToolProjectionPhase,
    call_id: &str,
    name: &str,
    raw_args: &str,
    model_call_id: &str,
    result: Option<Result<&str, &str>>,
) -> Option<ProjectedClientSideToolV2> {
    if call_id.is_empty() || name.is_empty() {
        return None;
    }
    match phase {
        ToolProjectionPhase::Started | ToolProjectionPhase::Partial => {
            Some(ProjectedClientSideToolV2::Call {
                tool_call_id: call_id.to_string(),
                name: name.to_string(),
                raw_args: raw_args.to_string(),
                model_call_id: (!model_call_id.is_empty()).then(|| model_call_id.to_string()),
                is_streaming: phase == ToolProjectionPhase::Partial,
            })
        }
        ToolProjectionPhase::Completed => {
            let result = result?;
            Some(match result {
                Ok(output) => ProjectedClientSideToolV2::Result {
                    tool_call_id: call_id.to_string(),
                    name: name.to_string(),
                    output: output.to_string(),
                    error: None,
                },
                Err(error) => ProjectedClientSideToolV2::Result {
                    tool_call_id: call_id.to_string(),
                    name: name.to_string(),
                    output: String::new(),
                    error: Some(error.to_string()),
                },
            })
        }
    }
}
