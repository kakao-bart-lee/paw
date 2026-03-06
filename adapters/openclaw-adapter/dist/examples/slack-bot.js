"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const src_1 = require("../src");
async function main() {
    const token = process.env.PAW_AGENT_TOKEN ?? 'dev-token';
    const adapter = new src_1.OpenClawAdapter(token, {
        serverUrl: process.env.PAW_SERVER_URL ?? 'ws://localhost:3000',
        channelConfig: { defaultChannel: 'slack' },
    });
    adapter.registerChannel(new src_1.SlackChannelHandler());
    await adapter.connect();
}
void main();
