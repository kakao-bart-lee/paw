# Contributing to Paw

We welcome contributions from the community! Whether you're fixing a bug, adding a feature, or improving documentation, your help is appreciated.

## Development Setup

### Prerequisites

- **Rust**: Latest stable version.
- **Flutter**: Latest stable version.
- **Docker**: For running PostgreSQL, MinIO, and NATS.
- **Node.js**: For TypeScript SDK and adapters.
- **Python**: 3.11+ for Python SDK.

### Running the Server

1. Start dependencies:
   ```bash
   docker-compose up -d
   ```
2. Run the server:
   ```bash
   cargo run --package paw-server
   ```

### Running the Client

1. Navigate to `paw-client`.
2. Run:
   ```bash
   flutter run
   ```

## Pull Request Process

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Write tests for your changes.
4. Ensure all tests pass:
   ```bash
   cargo test
   npm test
   pytest
   ```
5. Submit a pull request with a clear description of your changes.

## Code Style

- **Rust**: Follow `rustfmt` and `clippy`.
- **Flutter**: Follow the official Flutter style guide.
- **TypeScript**: Use Prettier and ESLint.
- **Python**: Use Black and Ruff.

## Security

If you find a security vulnerability, please do not open a public issue. Instead, email us at security@paw.im.
