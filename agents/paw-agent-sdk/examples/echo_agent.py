import asyncio

from paw_agent_sdk import ConversationContext, PawAgent


agent = PawAgent(token="paw_agent_your-token-here")


@agent.on_message
async def handle(ctx: ConversationContext) -> str:
    return f"Echo: {ctx.message.content}"


if __name__ == "__main__":
    asyncio.run(agent.run())
