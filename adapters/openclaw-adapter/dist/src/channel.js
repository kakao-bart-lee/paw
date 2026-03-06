"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SlackChannelHandler = exports.ChannelHandler = void 0;
class ChannelHandler {
}
exports.ChannelHandler = ChannelHandler;
class SlackChannelHandler extends ChannelHandler {
    name = 'slack';
    async handle(ctx) {
        return `[Slack] Echo: ${ctx.message.content}`;
    }
}
exports.SlackChannelHandler = SlackChannelHandler;
