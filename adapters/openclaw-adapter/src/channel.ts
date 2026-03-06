import { InboundContext } from './types';

export abstract class ChannelHandler {
  abstract name: string;
  abstract handle(ctx: InboundContext): Promise<string | AsyncGenerator<string>>;
}

export class SlackChannelHandler extends ChannelHandler {
  name = 'slack';

  async handle(ctx: InboundContext): Promise<string> {
    return `[Slack] Echo: ${ctx.message.content}`;
  }
}
