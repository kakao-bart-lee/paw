import { OpenClawAdapter, SlackChannelHandler } from '../src';

async function main(): Promise<void> {
  const token = process.env.PAW_AGENT_TOKEN ?? 'dev-token';

  const adapter = new OpenClawAdapter(token, {
    serverUrl: process.env.PAW_SERVER_URL ?? 'ws://localhost:3000',
    channelConfig: { defaultChannel: 'slack' },
  });

  adapter.registerChannel(new SlackChannelHandler());
  await adapter.connect();
}

void main();
