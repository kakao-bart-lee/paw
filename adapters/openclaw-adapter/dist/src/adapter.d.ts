import { AgentResponseMsg, E2eeConfig, InboundContext } from './types';
import { ChannelHandler } from './channel';
export interface ChannelConfig {
    defaultChannel?: string;
}
export interface AdapterOptions {
    serverUrl?: string;
    channelConfig?: ChannelConfig;
    agentId?: string;
    defaultFormat?: 'plain' | 'markdown';
    e2ee?: E2eeConfig;
}
export declare function parseInboundContext(raw: string): InboundContext | null;
export declare function serializeAgentResponse(conversationId: string, content: string, format?: 'plain' | 'markdown'): AgentResponseMsg;
export declare class OpenClawAdapter {
    private readonly token;
    private ws;
    private handlers;
    private readonly serverUrl;
    private readonly channelConfig;
    private readonly agentId;
    private readonly defaultFormat;
    private readonly e2eeConfig;
    constructor(token: string, options?: AdapterOptions);
    registerChannel(handler: ChannelHandler): this;
    connect(): Promise<void>;
    disconnect(): void;
    routeInboundContext(ctx: InboundContext): Promise<void>;
    private resolveHandler;
}
