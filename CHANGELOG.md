# Changelog

All notable changes to Paw Messenger will be documented in this file.

## [Phase 3] — Scale & Polish (Current)

### Added
- **Channel Support**: Broadcast messages to thousands of subscribers.
- **Marketplace**: Discover and install agents from a global marketplace.
- **Device Sync**: Efficiently sync multiple conversations across devices.
- **Moderation Tools**: Report users/messages and block/suspend accounts.
- **Backup & Restore**: Securely backup and restore conversation history.
- **Push Notifications**: Support for FCM and APNS.
- **Fly.io Deployment**: Production-ready deployment scripts.

### Improved
- **WebSocket Protocol**: Finalized v1 spec with `device_sync` support.
- **SDKs**: Enhanced Python and TypeScript SDKs with streaming and tool frames.
- **Documentation**: Comprehensive OpenAPI spec and quickstart guides.

## [Phase 2] — Agent & E2EE

### Added
- **End-to-End Encryption**: Signal protocol (X3DH + Double Ratchet) for secure messaging.
- **Agent Gateway**: Native support for AI agents via WebSocket.
- **Streaming Responses**: Real-time token-by-token streaming for agents.
- **Rich UI Blocks**: Support for cards, buttons, and interactive elements in messages.
- **Media Uploads**: Secure file and image sharing via MinIO.

### Improved
- **Rust Server**: Refactored for better performance and scalability.
- **Flutter Client**: Added support for E2EE and agent interactions.

## [Phase 1] — Core Messaging

### Added
- **User Authentication**: OTP-based login and registration.
- **Direct Messaging**: One-on-one conversations.
- **Group Messaging**: Multi-user conversations with group management.
- **WebSocket Transport**: Real-time message delivery.
- **PostgreSQL Persistence**: Reliable storage for users and messages.
- **Initial Flutter Client**: Basic messaging UI for iOS and Android.
