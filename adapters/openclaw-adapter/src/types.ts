export interface PawMessage {
  v: number;
}

export interface E2eeConfig {
  publicKey: string;
  privateKey: string;
}

export interface MessageReceived extends PawMessage {
  type: 'message_received';
  id: string;
  conversation_id: string;
  sender_id: string;
  content: string;
  format: 'plain' | 'markdown';
  seq: number;
  created_at: string;
}

export interface InboundContext extends PawMessage {
  type?: string;
  message: MessageReceived;
  conversation_id: string;
  recent_messages: MessageReceived[];
}

export interface AgentResponse extends PawMessage {
  conversation_id: string;
  content: string;
  format: 'plain' | 'markdown';
}

export type AgentStreamFrame =
  | {
      type: 'stream_start';
      v: number;
      conversation_id: string;
      agent_id: string;
      stream_id: string;
    }
  | {
      type: 'content_delta';
      v: number;
      stream_id: string;
      delta: string;
    }
  | {
      type: 'tool_start';
      v: number;
      stream_id: string;
      tool: string;
      label: string;
    }
  | {
      type: 'tool_end';
      v: number;
      stream_id: string;
      tool: string;
    }
  | {
      type: 'stream_end';
      v: number;
      stream_id: string;
      tokens: number;
      duration_ms: number;
    };

export type AgentResponseMsg = AgentResponse;
export type AgentStreamMsg = AgentStreamFrame;
