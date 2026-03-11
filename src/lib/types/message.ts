export type Message =
  | UserMessage
  | AssistantMessage
  | ReasoningStep
  | ToolApprovalRequest
  | ToolApprovalResponse
  | ToolExecution
  | SystemMessage
  | ErrorMessage;

export interface UserMessage {
  type: 'UserMessage';
  content: string;
}

export interface AssistantMessage {
  type: 'AssistantMessage';
  content: string;
  model: string;
}

export interface ReasoningStep {
  type: 'ReasoningStep';
  step: string;
  model: string;
  content: string;
}

export interface ToolApprovalRequest {
  type: 'ToolApprovalRequest';
  request_id: string;
  tool_name: string;
  parameters: Record<string, any>;
  reasoning: string;
}

export interface ToolApprovalResponse {
  type: 'ToolApprovalResponse';
  request_id: string;
  approved: boolean;
}

export interface ToolExecution {
  type: 'ToolExecution';
  tool_name: string;
  parameters: Record<string, any>;
  result: any;
}

export interface SystemMessage {
  type: 'SystemMessage';
  content: string;
}

export interface ErrorMessage {
  type: 'Error';
  message: string;
}
