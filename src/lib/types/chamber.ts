export interface ChamberState {
  session_id: string;
  orchestrator_plan: string;
  reasoning_responses: Record<string, string>;
  orchestrator_synthesis: string;
  next_action: 'continue_reasoning' | 'use_tools' | 'finalize';
  pending_tool_approval: ToolApproval | null;
  tool_results: ToolResult[];
  error: string | null;
  iteration_count: number;
}

export interface ToolApproval {
  tool_name: string;
  parameters: Record<string, any>;
  reasoning: string;
}

export interface ToolResult {
  tool_name: string;
  parameters: Record<string, any>;
  result: any;
  timestamp: number;
}
