# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Markdown media image management: prepare, complete, list, delete, and a
  high-level hash-and-upload helper.
- `message::mention_all_members()` for the QQ group `<qqbot-at-everyone />` tag.
- Production OpenAPI domain updated to `https://api.bot.qq.com`; sandbox remains
  `https://sandbox.api.sgroup.qq.com`.
- API v2 group management: group info/state, members, blacklist, bulk removal,
  join requests and approval, mute settings.
- Join approval strategies, global C2C menu, command panels, share links.
- Rich-media URL/binary/chunked uploads and streaming C2C messages.
- API v2 events and intents: `GROUP_MESSAGE_CREATE`, robot add/remove, join
  requests, group member changes, group/C2C receive state changes.
- `GET /gateway` and `GatewayUrl`.

### Changed

- Raised MSRV to Rust 1.88 because the latest `wiremock` dev-dependency uses
  let-chains, which were stabilized in 1.88.
- Synced message, user, attachment, card, ARK, and keyboard models with API v2
  fields (`group_openid`, `message_type`, `ark_data`, `msg_elements`,
  `message_scene.ext`, `is_wakeup`, `card`, `unsupport_tips`, `reply`, `anchor`).
- Updated channel type constants to the current API values (`10005` live,
  `10006` application, `10007` forum).
- Added interaction types 11-20 and resolved authorization/story/feedback data.

## [0.1.0] - 2026-11-10

### Added

- Async OpenAPI v1 client with the complete public endpoint set from
  `tencent-connect/botgo` v0.2.1.
- `TokenSource` trait and `QqBotTokenSource` with caching, concurrent refresh
  deduplication, cancellation-safe background refresh, and secret redaction.
- WebSocket gateway client with sharding, Identify/Resume, heartbeats,
  exponential backoff, and typed event dispatch.
- Typed `Event` enum covering all upstream gateway events, including forum,
  C2C, group, subscription, and AIO events.
- Ed25519 webhook signing and verification, framework-agnostic callback
  processing, URL validation, heartbeat ACK, and dispatch ACK.
- Full `dto` model port, request options, pagers, message helpers, and inline
  search simulator.
- Integration tests using Wiremock and an in-process WebSocket gateway.
- Chinese README, API mapping, examples, CI workflow, Apache-2.0 license.
