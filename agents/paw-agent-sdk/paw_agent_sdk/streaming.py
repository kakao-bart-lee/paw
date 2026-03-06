from collections.abc import AsyncGenerator, Awaitable, Callable
from typing import ParamSpec


P = ParamSpec("P")


class StreamingResponse:
    """Async generator wrapper for streaming agent responses."""

    def __init__(self, generator: AsyncGenerator[str, None]):
        self._gen = generator

    def __aiter__(self):
        return self._gen.__aiter__()


def stream(
    func: Callable[P, AsyncGenerator[str, None]],
) -> Callable[P, Awaitable[StreamingResponse]]:
    """Decorator to mark an async generator as a streaming response."""

    async def wrapper(*args: P.args, **kwargs: P.kwargs) -> StreamingResponse:
        return StreamingResponse(func(*args, **kwargs))

    return wrapper
