import { InboundContext } from './types';
export declare abstract class ChannelHandler {
    abstract name: string;
    abstract handle(ctx: InboundContext): Promise<string | AsyncGenerator<string>>;
}
export declare class SlackChannelHandler extends ChannelHandler {
    name: string;
    handle(ctx: InboundContext): Promise<string>;
}
