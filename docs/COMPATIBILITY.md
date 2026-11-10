# Go 到 Rust 兼容性说明

基线：`tencent-connect/botgo` commit
`fe31c0dfe469001e0f783d2f07e7de7bd08b403f`（tag `v0.2.1`）。

## 包级映射

| Go 包/文件 | Rust 模块 | 说明 |
| --- | --- | --- |
| `botgo.go` | `openapi`, `gateway` | 构造与版本选择由具体类型和 builder 完成 |
| `register.go` | `gateway`, `tracing` | 全局注册改为显式事件流和 subscriber |
| `session_manager.go` | `gateway` | 内置分片/重连，不再暴露全局 manager |
| `constant` | `openapi`, `token` | 域名、Header、Token URL 常量 |
| `dto` | `models` | 全部公开结构体 |
| `dto/keyboard` | `models::keyboard` | 按钮模型 |
| `dto/message` | `message` | `ETLInput`、`Mention*`、`ParseCommand` |
| `errs` | `error` | `Error`、`ApiError`、close code |
| `event` | `event`, `gateway` | 强类型 `Event` 与 `GatewayEvent` |
| `interaction/search` | `search` | `simulate_search` |
| `interaction/signature` | `signature` | Ed25519 |
| `interaction/webhook` | `webhook` | 框架无关的 handler 逻辑 |
| `log` | `tracing` | 不提供全局 logger 替换 |
| `openapi` | `openapi` | `OpenApi` / `OpenApiBuilder` |
| `openapi/options` | `openapi::RequestOptions` | 自定义 URL、`hidetip` |
| `openapi/v1` | `openapi` | 合并为单一 `OpenApi` 客户端，覆盖 v1 与 v2 文档接口 |
| `sessions/local` | `gateway` | 本地分片管理 |
| `sessions/remote` | 未实现 | Redis 分布式 session manager |
| `token` | `token` | `TokenSource`、`QqBotTokenSource` |
| `version` | `botgo::VERSION` | `User-Agent` 使用 |

## OpenAPI 方法映射

| Go | Rust |
| --- | --- |
| `WS` | `ws` |
| `Me`, `MeGuilds` | `me`, `me_guilds` |
| `Message`, `Messages` | `message`, `messages` |
| `PostMessage`, `PatchMessage`, `RetractMessage` | `post_message`, `patch_message`, `retract_message` |
| `PostSettingGuide` | `post_setting_guide` |
| `PostGroupMessage`, `PostC2CMessage` | `post_group_message`, `post_c2c_message` |
| `RetractC2CMessage`, `RetractGroupMessage` | `retract_c2c_message`, `retract_group_message` |
| `Guild`, `GuildMember`, `GuildMembers` | `guild`, `guild_member`, `guild_members` |
| `GuildRoleMembers`, `DeleteGuildMember` | `guild_role_members`, `delete_guild_member` |
| `GuildMute`, `MemberMute`, `MultiMemberMute` | `guild_mute`, `member_mute`, `multi_member_mute` |
| `Channel`, `Channels`, `PostChannel` | `channel`, `channels`, `post_channel` |
| `PatchChannel`, `DeleteChannel` | `patch_channel`, `delete_channel` |
| `CreatePrivateChannel`, `ListVoiceChannelMembers` | `create_private_channel`, `list_voice_channel_members` |
| `ChannelPermissions`, `PutChannelPermissions` | `channel_permissions`, `put_channel_permissions` |
| `ChannelRolesPermissions`, `PutChannelRolesPermissions` | `channel_roles_permissions`, `put_channel_roles_permissions` |
| `PostAudio`, `PutMic`, `DeleteMic` | `post_audio`, `put_mic`, `delete_mic` |
| `Roles`, `PostRole`, `PatchRole`, `DeleteRole` | `roles`, `post_role`, `patch_role`, `delete_role` |
| `MemberAddRole`, `MemberDeleteRole` | `member_add_role`, `member_delete_role` |
| `CreateDirectMessage`, `PostDirectMessage` | `create_direct_message`, `post_direct_message` |
| `RetractDMMessage`, `PostDMSettingGuide` | `retract_direct_message`, `post_direct_message_setting_guide` |
| `CreateChannelAnnounces`, `DeleteChannelAnnounces`, `CleanChannelAnnounces` | `create_channel_announce`, `delete_channel_announce`, `clean_channel_announces` |
| `CreateGuildAnnounces`, `DeleteGuildAnnounces`, `CleanGuildAnnounces` | `create_guild_announce`, `delete_guild_announce`, `clean_guild_announces` |
| `ListSchedules`, `GetSchedule`, `CreateSchedule` | `list_schedules`, `get_schedule`, `create_schedule` |
| `ModifySchedule`, `DeleteSchedule` | `modify_schedule`, `delete_schedule` |
| `GetAPIPermissions`, `RequireAPIPermissions` | `get_api_permissions`, `require_api_permissions` |
| `AddPins`, `DeletePins`, `CleanPins`, `GetPins` | `add_pin`, `delete_pin`, `clean_pins`, `get_pins` |
| `CreateMessageReaction`, `DeleteOwnMessageReaction` | `create_message_reaction`, `delete_own_message_reaction` |
| `GetMessageReactionUsers` | `get_message_reaction_users` |
| `PutInteraction` | `put_interaction` |
| `CreateSession`, `CheckSessions`, `SessionList`, `RemoveSession` | `create_session`, `check_sessions`, `session_list`, `remove_session` |
| `GetMessageSetting` | `get_message_setting` |
| `Transport` | `transport` |
| `TraceID` | `trace_id` |

## API v2 文档增量（2026-11）

Go SDK 当前未覆盖 `https://bot.q.qq.com/wiki/develop/api-v2/` 中的群管理、菜单、
面板、上传和流式接口；Rust 版已按该文档补齐。当前文档列出的 47 个 API 端点均有对应方法。

| 文档能力 | Rust 方法 |
| --- | --- |
| 通用 WSS 接入点 `GET /gateway` | `gateway` |
| 群信息 / 机器人群内状态 | `group_info`, `group_bot_state` |
| 群成员列表 / 成员详情 | `group_members`, `group_member` |
| 群黑名单查询 / 操作 | `group_blacklist`, `update_group_blacklist` |
| 群成员批量移除 | `batch_remove_group_members` |
| 入群申请列表 / 审批 | `group_join_requests`, `approve_group_join_request` |
| 群禁言查询 / 成员禁言 | `group_restrict_chat_setting`, `set_group_member_mutes` |
| 入群自动审批策略 CRUD / 执行 / 白名单 | `join_approval_strategies`, `create_join_approval_strategy`, `update_join_approval_strategy`, `delete_join_approval_strategy`, `execute_join_approval_strategy`, `update_join_approval_whitelist` |
| 全局菜单查询 / 修改 | `menu`, `update_menu` |
| 指令面板查询 / 创建 / 详情 / 修改 / 删除 / 关联对象 | `panels`, `create_panel`, `panel`, `update_panel`, `delete_panel`, `update_panel_targets` |
| 机器人分享链接 | `generate_url_link` |
| 群/单聊富媒体 URL、二进制、分片上传 | `upload_group_file`, `upload_user_file`, `prepare_group_upload`, `finish_group_upload_part`, `prepare_user_upload`, `finish_user_upload_part` |
| 流式单聊消息 | `stream_c2c_message` |

同时补齐的 v2 事件与 Intent：

| 文档事件 | Intent | Rust `Event` |
| --- | --- | --- |
| `GROUP_MESSAGE_CREATE` | `1 << 25` | `GroupMessageCreate` |
| `GROUP_ADD_ROBOT` / `GROUP_DEL_ROBOT` | `1 << 25` | `GroupAddRobot` / `GroupDelRobot` |
| `GROUP_JOIN_REQUEST` | `1 << 24` | `GroupJoinRequest` |
| `GROUP_MEMBER_ADD` / `GROUP_MEMBER_REMOVE` | `1 << 24` | `GroupMemberAdd` / `GroupMemberRemove` |
| `GROUP_MSG_RECEIVE` / `GROUP_MSG_REJECT` | `1 << 25` | `GroupMsgReceive` / `GroupMsgReject` |
| `C2C_MSG_RECEIVE` / `C2C_MSG_REJECT` | `1 << 25` | `C2cMsgReceive` / `C2cMsgReject` |

`Intents::GROUP_AND_C2C_EVENT` 是 `GROUP_MESSAGES` 的别名；成员事件需额外订阅
`Intents::GROUP_MEMBER_EVENT`。

## 有意差异

1. **无全局可变状态**：Go 版的 `openapi.DefaultImpl`、`event.DefaultHandlers`、`log.DefaultLogger`、
   `websocket.ClientImpl` 都是包级变量。Rust 版将客户端、TokenSource、Gateway、事件流都作为值传递，
   便于多机器人、多租户和并行测试。
2. **事件模型**：`event.RegisterHandlers` 的运行时类型断言改为 `GatewayEvent`/`Event` 枚举匹配。
   未知事件保留为 `Event::Unknown`，不会静默丢弃。
3. **错误处理**：所有失败返回 `Result<T, Error>`；API 错误保留 HTTP status、`code`、`err_code`、
   `trace_id` 和原始 body。
4. **分页参数**：Go 使用字符串字段；Rust 使用 `Option<u32>`/`Option<String>` builder，
   避免空字符串被序列化。
5. **消息单条查询**：Go 版的 `Message` 使用列表 URL；Rust 版使用
   `/channels/{channel_id}/messages/{message_id}`。
6. **签名验证**：`signature::verify` 返回 `Result<()>`，失败为 `Error::Signature`；
   提供 `is_valid` 作为布尔便捷函数。
7. **Token 刷新**：`start_refresh` 返回 `TokenRefresh` guard，drop 即取消；
   Go 版依赖 `context.Context` 取消。
8. **Redis 分布式 session manager**：未复刻。该能力与部署拓扑强相关，Rust 版保留
   `Gateway::with_shard` 供业务层自行协调分片。
9. **Logger**：使用 `tracing`，不提供日志实现；应用自行安装 subscriber。

## 未移植的公开标识符

- `openapi.APIVersion` / `SelectOpenAPIVersion`：Rust 版只有 v1 具体实现。
- `openapi.Setup` / `OpenAPI.Version` / `SetDebug`：由 `OpenApiBuilder` 和 `tracing` 替代。
- `websocket.WebSocket` 接口与 `Register`：`Gateway` 是具体实现；如需自定义传输层，可基于
  `tokio-tungstenite` 实现独立 client。
- `sessions/remote`、`sessions/remote/lock`：见上。
- `log.Logger` 接口、`SetLogger`：见上。
- `version.String`：使用 `format!("BotGoSDK/{}", botgo::VERSION)`。
