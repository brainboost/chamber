use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    UserMessage {
        content: String,
    },
    AssistantMessage {
        content: String,
        model: String,
    },
    ReasoningStep {
        step: String,
        model: String,
        content: String,
    },
    ToolApprovalRequest {
        request_id: String,
        tool_name: String,
        parameters: serde_json::Value,
        reasoning: String,
    },
    ToolApprovalResponse {
        request_id: String,
        approved: bool,
    },
    ToolExecution {
        tool_name: String,
        parameters: serde_json::Value,
        result: serde_json::Value,
    },
    SystemMessage {
        content: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarRequest {
    pub session_id: String,
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}
