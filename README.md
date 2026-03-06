# Paw Messenger

Paw is a secure, agent-first messaging platform designed for the modern AI era. It combines end-to-end encryption (E2EE) with a powerful agent ecosystem, allowing users to interact with AI agents as easily as they do with friends.

## Features

- **Secure Messaging**: End-to-end encryption using the Signal protocol (X3DH + Double Ratchet).
- **Agent-First**: Native support for AI agents with streaming responses and rich UI blocks.
- **Channels**: Broadcast messages to thousands of subscribers.
- **Marketplace**: Discover and install agents from a global marketplace.
- **Cross-Platform**: Flutter-based client for iOS, Android, and Desktop.
- **Developer Friendly**: Comprehensive SDKs for Python and TypeScript.

## Architecture

```text
                                  +-----------------------+
                                  | PostgreSQL/MinIO/NATS |
                                  +-----------^-----------+
                                              |
+----------------+       +--------------------+--------------------+
| Flutter Client | <---> |             paw-server                  |
+----------------+       | (REST, WS, Auth, Media, Agent Gateway)  |
                         +--------------------+--------------------+
                                              |
                                              v (WS /agent/ws)
                                     +-----------------+
                                     |  Python/TS SDK  |
                                     +--------+--------+
                                              |
                                              v
                                     +-----------------+
                                     | OpenClaw Adapter|
                                     +-----------------+
```

## Quickstart

### Server

1. Clone the repository.
2. Run `docker-compose up -d` to start dependencies (PostgreSQL, MinIO, NATS).
3. Run `cargo run --package paw-server` to start the server.

### Client

1. Navigate to `paw-client`.
2. Run `flutter run`.

## Documentation

- [API Reference](docs/api/openapi.yaml)
- [WebSocket Protocol](docs/protocol-v1.md)
- [Python SDK Quickstart](docs/sdk/python-quickstart.md)
- [TypeScript SDK Quickstart](docs/sdk/typescript-quickstart.md)
- [Architecture Deep Dive](docs/ARCHITECTURE.md)

## License

MIT License. See [LICENSE](LICENSE) for details.
