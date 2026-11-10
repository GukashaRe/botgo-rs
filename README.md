# botgo-rs

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

QQ 机器人 OpenAPI 与事件网关的 Rust SDK，是腾讯官方 Go SDK
[`tencent-connect/botgo`](https://github.com/tencent-connect/botgo) 的本地 Rust 复刻。

- 源码基线：`fe31c0dfe469001e0f783d2f07e7de7bd08b403f`（`v0.2.1`，2024-12-18）
- Rust Edition 2024，MSRV 1.88
- 全部公开网络能力均为 `async`，基于 Tokio / Reqwest
- 不持有全局可变状态；客户端、TokenSource、Gateway 都是显式值
- 使用 typed event stream 取代 Go 版的全局回调注册表
- 使用 `tracing` 取代可替换的 logger 全局变量
- 通过 `cargo fmt`、`cargo clippy -D warnings`、`cargo test`、`cargo doc`

## 功能

| 模块 | 内容 |
| --- | --- |
| `openapi` | v1 频道接口 + v2 群管理、入群审批、菜单、指令面板、素材管理、富媒体上传、流式消息等全部 v2 接口 |
| `token` | QQ Bot `appId`/`clientSecret` TokenSource，缓存、并发去重、后台提前刷新 |
| `gateway` | WebSocket 分片、Hello/Identify/Resume、心跳、断线重连、会话恢复，含群/C2C 生命周期事件 |
| `event` | 全部事件类型的强类型枚举，未知事件保留原始 JSON |
| `models` | 与 Go `dto` 包对应的请求/响应模型，缺失字段使用默认值 |
| `signature` | Webhook/互动回调的 Ed25519 签名与验签 |
| `webhook` | URL 校验、心跳 ACK、事件 ACK，框架无关 |
| `search` | 内联搜索模拟器 |
| `message` | `ETLInput`、@ 提及、子频道提及、emoji、命令解析 |

## 环境要求

- Rust `1.88+`（Edition 2024）
- Windows 编译原生依赖时需要 VS 2022 Build Tools，并勾选“使用 C++ 的桌面开发”
- Linux/macOS 使用默认 `rustls` 即可；也可以切换 `native-tls`
- 一个 QQ 开放平台机器人应用，并已开通沙盒环境
- 测试群聊时，机器人必须已加入该沙盒群；测试私聊时，用户必须先添加机器人好友

Windows 下建议始终通过包装脚本构建，避免 `cc` 选到 VS 2019 或 `%TEMP%` 权限问题：

```powershell
.\run-vs2022.ps1 check
```

## 安装

本仓库是本地复刻项目，使用路径依赖：

```toml
[dependencies]
botgo = { path = "../botgo-rs" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

如需发布到 crates.io，可改用版本号依赖。默认启用 `rustls-tls`。如需系统 TLS，
可关闭默认特性并启用 `native-tls`：

```toml
botgo = { version = "0.1", default-features = false, features = ["native-tls"] }
```

## 快速开始

```rust,no_run
use std::sync::Arc;

use botgo::models::MessageToCreate;
use botgo::token::{Credentials, QqBotTokenSource};
use botgo::{OpenApi, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let app_id = std::env::var("QQ_BOT_APP_ID")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_ID is not set"))?;
    let app_secret = std::env::var("QQ_BOT_APP_SECRET")
        .map_err(|_| botgo::Error::config("QQ_BOT_APP_SECRET is not set"))?;

    let token_source = Arc::new(QqBotTokenSource::new(Credentials::new(&app_id, app_secret)));
    let _refresh = token_source.start_refresh();
    let api = OpenApi::new(app_id, token_source)?;

    let message = api
        .post_message(
            "channel-id",
            &MessageToCreate {
                content: "hello from Rust".to_owned(),
                ..MessageToCreate::default()
            },
        )
        .await?;

    println!("sent {}", message.id);
    Ok(())
}
```

更多示例：

```bash
cargo run --example echo
cargo run --example sandbox_check
cargo run --example send_message
cargo run --example gateway
cargo run --example webhook
```

## 最小 Echo 示例（沙盒）

[`examples/echo.rs`](examples/echo.rs) 是一个最小可运行的沙盒 Echo 机器人：

- 固定使用 `OpenApi::sandbox`；
- 订阅 `Intents::GROUP_MESSAGES`；
- 收到 `GROUP_AT_MESSAGE_CREATE` 或 `C2C_MESSAGE_CREATE` 后，把收到的文本原样回复；
- 使用 `msg_id` 做被动回复，避免主动消息频控。

### 环境变量

| 变量 | 必需 | 用途 |
| --- | --- | --- |
| `QQ_BOT_APP_ID` | 是 | 机器人 AppID |
| `QQ_BOT_APP_SECRET` | 是 | 机器人 AppSecret，只在本地环境变量中设置 |
| `QQ_BOT_GROUP_OPENID` | 测试群聊时 | 沙盒群 OpenID，`sandbox_check` 和 live test 使用 |
| `QQ_BOT_USER_OPENID` | 测试私聊时 | 沙盒用户 OpenID，`sandbox_check` 和 live test 使用 |

`group_openid` / `user_openid` 是平台下发的 OpenID，不是 QQ 群号或 QQ 号。
启动 `echo` 后，在沙盒群 @机器人、或用另一个账号私聊机器人，控制台会打印：

```text
group <group_openid>: <content>
c2c <user_openid>: <content>
```

把尖括号后的 ID 填入对应环境变量即可。

PowerShell 运行：

```powershell
$env:QQ_BOT_APP_ID = "你的 AppID"
$env:QQ_BOT_APP_SECRET = "你的 AppSecret"
.\run-vs2022.ps1 run --example echo
```

Bash 运行：

```bash
export QQ_BOT_APP_ID="你的 AppID"
export QQ_BOT_APP_SECRET="你的 AppSecret"
cargo run --example echo
```

示例不会把密钥写入源码或配置文件。请把机器人加入沙盒群，并在沙盒环境 @机器人或私聊它进行测试。

### Windows 强制使用 VS 2022

如果机器同时安装了 VS 2019 和 VS 2022，`cc`/`aws-lc-sys` 可能仍通过
`vswhere -latest` 选中尚未注册完成的 VS 2019。可以直接使用项目自带包装脚本，
它会显式选择 VS 2022 的 `cl.exe`、加载 `vcvars64.bat`，并把 `TMP/TEMP` 指到
项目内可写目录：

```powershell
.\run-vs2022.ps1 run --example echo
```

PowerShell 执行策略阻止脚本时：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\run-vs2022.ps1 run --example echo
```

脚本也接受其他 Cargo 参数，例如：

```powershell
.\run-vs2022.ps1 check --example echo
.\run-vs2022.ps1 test --all-targets
```

## 沙盒群聊/私聊主动消息测试

[`examples/sandbox_check.rs`](examples/sandbox_check.rs) 用于验证沙盒 OpenAPI 和主动消息：

1. 获取 AccessToken；
2. 调用 `/users/@me` 验证沙盒鉴权；
3. 如果设置了 `QQ_BOT_GROUP_OPENID`，向群发送一条主动测试消息；
4. 如果设置了 `QQ_BOT_USER_OPENID`，向用户发送一条主动测试消息。

```powershell
$env:QQ_BOT_APP_ID = "你的 AppID"
$env:QQ_BOT_APP_SECRET = "你的 AppSecret"
$env:QQ_BOT_GROUP_OPENID = "沙盒群 OpenID"
$env:QQ_BOT_USER_OPENID = "沙盒用户 OpenID"

.\run-vs2022.ps1 run --example sandbox_check
```

只测试群聊时删除 `QQ_BOT_USER_OPENID`，只测试私聊时删除
`QQ_BOT_GROUP_OPENID`。成功时输出类似：

```text
sandbox connected as <bot name> (<bot id>)
group message sent: id=...
c2c message sent: id=...
sandbox check passed
```

注意：`sandbox_check` 发送的是**主动消息**，会消耗主动消息额度；群聊还要求机器人
已在群内，私聊还要求用户已添加机器人好友并允许主动消息。被动回复请用 `echo`。

## 自动化测试

默认测试全部使用本地 Mock HTTP / WebSocket，不需要真实凭证：

```bash
cargo test --all-targets
```

Windows 下建议用包装脚本：

```powershell
.\run-vs2022.ps1 test --all-targets
```

真实沙盒测试是 opt-in 的 `#[ignore]` 测试，会发送主动消息：

```powershell
$env:QQ_BOT_APP_ID = "你的 AppID"
$env:QQ_BOT_APP_SECRET = "你的 AppSecret"
$env:QQ_BOT_GROUP_OPENID = "沙盒群 OpenID"
$env:QQ_BOT_USER_OPENID = "沙盒用户 OpenID"

.\run-vs2022.ps1 test --test sandbox_live -- --ignored --nocapture
```

只跑其中一个：

```powershell
.\run-vs2022.ps1 test --test sandbox_live sandbox_group_message -- --ignored --nocapture
.\run-vs2022.ps1 test --test sandbox_live sandbox_c2c_message -- --ignored --nocapture
```

对应测试文件：[tests/sandbox_live.rs](tests/sandbox_live.rs)。

## WebSocket 事件网关

```rust,no_run
use std::sync::Arc;

use botgo::gateway::{Gateway, GatewayEvent};
use botgo::models::Intents;
use botgo::token::{Credentials, QqBotTokenSource};
use botgo::{Event, OpenApi, Result};
use futures_util::StreamExt;

# async fn run() -> Result<()> {
let token_source = Arc::new(QqBotTokenSource::new(Credentials::new("app-id", "secret")));
let _refresh = token_source.start_refresh();
let api = OpenApi::new("app-id", Arc::clone(&token_source))?;

let intents = Intents::GUILDS | Intents::GUILD_AT_MESSAGE | Intents::GROUP_MESSAGES;
let mut events = Gateway::new(api, token_source, intents).events()?;

while let Some(item) = events.next().await {
    match item? {
        GatewayEvent::Ready { data, .. } => println!("ready as {}", data.user.username),
        GatewayEvent::Dispatch { event: Event::AtMessageCreate(message), .. } => {
            println!("{}", message.content);
        }
        GatewayEvent::Reconnecting { reason, delay_seconds } => {
            eprintln!("reconnect in {delay_seconds}s: {reason}");
        }
        _ => {}
    }
}
# Ok(())
# }
```

`Gateway` 会：

1. 调用 `/gateway/bot` 获取 WSS 地址、分片数和启动频控；
2. 按 `max_concurrency` 计算分片启动间隔；
3. 每个分片独立执行 Hello → Identify/Resume → 心跳循环；
4. 在可恢复错误上自动重连，在机器人被封禁/下架时向事件流发送终止错误。

事件流是冷启动的后台任务；丢弃 `EventStream` 会停止全部分片任务。

## QQ 群与私聊（C2C）

支持 QQ 群消息和 C2C 私聊：

| 场景 | Intent | 事件 | 发送方法 |
| --- | --- | --- | --- |
| QQ 群 @机器人 | `Intents::GROUP_MESSAGES` | `GROUP_AT_MESSAGE_CREATE` | `post_group_message` |
| C2C 私聊 | `Intents::GROUP_MESSAGES` | `C2C_MESSAGE_CREATE` | `post_c2c_message` |
| 好友添加/删除 | `Intents::GROUP_MESSAGES` | `FRIEND_ADD` / `FRIEND_DEL` | 根据 `openid` 发 C2C |
| 进入 AIO | `Intents::ENTER_AIO` | `ENTER_AIO` | — |

```rust,no_run
use botgo::models::{Intents, MessageToCreate};
use botgo::{Event, Gateway, GatewayEvent};
use futures_util::StreamExt;

# async fn run(api: botgo::OpenApi) -> botgo::Result<()> {
let mut events = Gateway::from_api(api.clone(), Intents::GROUP_MESSAGES).events()?;
while let Some(item) = events.next().await {
    match item? {
        GatewayEvent::Dispatch { event: Event::GroupAtMessageCreate(message), .. } => {
            let reply = MessageToCreate {
                content: "收到群消息".to_owned(),
                msg_id: message.id.clone(),
                ..MessageToCreate::default()
            };
            api.post_group_message(&message.group_id, reply).await?;
        }
        GatewayEvent::Dispatch { event: Event::C2cMessageCreate(message), .. } => {
            let Some(author) = message.author else { continue };
            let reply = MessageToCreate {
                content: "收到私聊".to_owned(),
                msg_id: message.id,
                ..MessageToCreate::default()
            };
            api.post_c2c_message(&author.id, reply).await?;
        }
        _ => {}
    }
}
# Ok(())
# }
```

群/C2C 富媒体发送到 `/v2/groups/{group_id}/files` 或 `/v2/users/{user_id}/files`；
普通消息发送到对应的 `/messages`。撤回使用 `retract_group_message` /
`retract_c2c_message`。

注意：`DirectMessage` / `post_direct_message` 是**频道私信**（Guild DM channel），
与 QQ 的 C2C 私聊不是同一个接口；C2C 请使用 `post_c2c_message`。

## API v2 群管理与高级能力

已按 2026-11 的 [QQ 机器人 API v2 文档](https://bot.q.qq.com/wiki/develop/api-v2/) 补齐该文档列出的 51 个 API 端点：

| 能力 | 方法 |
| --- | --- |
| 群信息 / 机器人群内状态 | `group_info`, `group_bot_state` |
| 群成员 / 成员详情 | `group_members`, `group_member` |
| 群黑名单 / 批量移除 | `group_blacklist`, `update_group_blacklist`, `batch_remove_group_members` |
| 入群申请列表 / 审批 | `group_join_requests`, `approve_group_join_request` |
| 群禁言状态 / 成员禁言 | `group_restrict_chat_setting`, `set_group_member_mutes` |
| 入群自动审批策略 | `join_approval_strategies`, `create_join_approval_strategy`, `update_join_approval_strategy`, `delete_join_approval_strategy`, `execute_join_approval_strategy`, `update_join_approval_whitelist` |
| 全局自定义菜单 | `menu`, `update_menu` |
| 指令面板 | `panels`, `create_panel`, `panel`, `update_panel`, `delete_panel`, `update_panel_targets` |
| 分享链接 | `generate_url_link` |
| 素材管理（Markdown 图片） | `upload_media_image`, `prepare_media_image`, `complete_media_image`, `media_images`, `delete_media_image` |
| 富媒体上传 | `upload_group_file`, `upload_user_file`, `prepare_group_upload`, `finish_group_upload_part`, `prepare_user_upload`, `finish_user_upload_part` |
| 流式单聊 | `stream_c2c_message` |
| 通用网关 | `gateway`（`GET /gateway`） |

对应事件与 Intent：

- 群成员事件：`Intents::GROUP_MEMBER_EVENT`（`1 << 24`）
- 群/C2C 消息与生命周期：`Intents::GROUP_MESSAGES` 或别名 `Intents::GROUP_AND_C2C_EVENT`（`1 << 25`）
- 新增事件：`GROUP_MESSAGE_CREATE`、`GROUP_ADD_ROBOT`、`GROUP_DEL_ROBOT`、`GROUP_JOIN_REQUEST`、`GROUP_MEMBER_ADD`、`GROUP_MEMBER_REMOVE`、`GROUP_MSG_RECEIVE`、`GROUP_MSG_REJECT`、`C2C_MSG_RECEIVE`、`C2C_MSG_REJECT`

消息模型也已同步 v2 字段：`group_openid`、`message_type`、`ark_data`、`msg_elements`、`message_scene.ext`、语音附件、图文卡片（`msg_type=8`）、`is_wakeup`、`force_verify_image_resource`，以及按钮的 `unsupport_tips` / `reply` / `anchor`。

Markdown 中可使用 `message::mention_all_members()` 生成 `<qqbot-at-everyone />`（需机器人为群管理员，每个群每天 1 次）。生产 OpenAPI 域名已切换到 `https://api.bot.qq.com`，沙盒仍使用 `https://sandbox.api.sgroup.qq.com`。

`GatewayEvent::Dispatch { event, event_id, seq }` 会保留事件外层 `id`，因此
`GROUP_ADD_ROBOT`、`GROUP_MSG_RECEIVE` 等事件可直接用 `event_id` 做被动回复；
Webhook 的 `WebhookResponse` 也提供 `event_id` 和 `seq`。

## Webhook

SDK 不绑定 Axum/Actix 等框架。框架处理器只需把原始 body 和两个签名头传给
`webhook::process`：

```rust,no_run
use botgo::webhook;

fn handle(
    secret: &str,
    signature: &str,
    timestamp: &str,
    body: &[u8],
) -> Result<(u16, Vec<u8>), botgo::Error> {
    let response = webhook::process(secret, signature, timestamp, body)?;
    Ok((response.status, response.body))
}
```

`process` 会完成验签、URL 校验、心跳 ACK、事件解析和事件 ACK。事件解析失败时返回
HTTP 200 + 负 ACK，让平台重试；签名失败返回 `Error::Signature`，调用方应回 401。

## Go API 对照

| Go | Rust |
| --- | --- |
| `botgo.NewOpenAPI(appID, tokenSource)` | `OpenApi::new(app_id, token_source)?` |
| `botgo.NewSandboxOpenAPI` | `OpenApi::sandbox(...)` / `OpenApiBuilder::sandbox(true)` |
| `.WithTimeout(d)` | `OpenApiBuilder::timeout(d)` |
| `.SetDebug(true)` | 使用 `tracing` 的 `DEBUG` 级别 |
| `token.NewQQBotTokenSource` | `QqBotTokenSource::new(Credentials)` |
| `token.StartRefreshAccessToken` | `Arc::new(source).start_refresh()`，返回可取消的 guard |
| `event.RegisterHandlers` | `Gateway::new(..., intents).events()` |
| `event.DefaultHandlers` | `GatewayEvent::Dispatch(Event)` 流 |
| `dto.WSPayload` | `models::WsPayload` |
| `dto.Intent` 位掩码 | `models::Intents` |
| `dto.MessageToCreate` | `models::MessageToCreate` |
| `message.ETLInput` | `message::etl_input` |
| `openapi.OpenAPI` 接口 | `OpenApi` 具体类型（Rust 不需要运行时版本注册表） |
| `log.Logger` | `tracing` span/event |
| `sessions/local` | `Gateway` 内置分片与重连管理 |
| `sessions/remote`（Redis） | 未复刻；需要分布式分片时建议由业务层协调多个 `Gateway::with_shard` 实例 |

除方法名改为 `snake_case`、错误改为 `Result`、可选参数改为 builder/`Option` 外，
核心请求路径、主要 JSON 字段、状态码判定和重连语义与 Go 版保持一致。修复了 Go 版
`Message` 单条查询误用列表路径的问题，Rust 版使用
`/channels/{channel_id}/messages/{message_id}`。

完整的包、方法和行为差异见 [COMPATIBILITY.md](docs/COMPATIBILITY.md)。

## 依赖

直接依赖均使用当前最新稳定版本（2026-11-10 查询）：

| crate | 版本 |
| --- | --- |
| `tokio` | 1.54.1 |
| `reqwest` | 0.13.5 |
| `tokio-tungstenite` | 0.30.0 |
| `serde` / `serde_json` | 1.0.229 / 1.0.154 |
| `thiserror` | 2.0.21 |
| `async-trait` | 0.1.92 |
| `ed25519-dalek` | 3.0.0 |
| `futures-util` | 0.3.34 |
| `tracing` | 0.1.44 |
| `hex` | 0.4.3 |
| `md-5` / `sha1` | 0.11.0 / 0.11.0 |

`Cargo.lock` 会锁定实际解析到的完整依赖树。升级依赖：

```bash
cargo update
cargo test --all-targets
```

## 项目结构

```text
src/
  openapi.rs          OpenAPI v1/v2 客户端
  token.rs            AccessToken 获取与刷新
  gateway.rs          WebSocket 网关、分片、重连
  event.rs            强类型事件分发
  signature.rs        Ed25519 签名
  webhook.rs          Webhook 校验与 ACK
  message.rs          消息文本工具
  models/             全部 DTO / 事件数据模型
examples/
  echo.rs             沙盒群聊/私聊 Echo
  sandbox_check.rs    沙盒主动消息连通性测试
  send_message.rs     频道消息发送
  gateway.rs          网关事件监听
  webhook.rs          Webhook 处理
tests/
  api_v2.rs           API v2 Mock 测试
  gateway.rs          本地 WebSocket 网关测试
  openapi.rs          OpenAPI Mock 测试
  qq_group_c2c.rs     群聊/私聊 Mock 测试
  sandbox_live.rs     opt-in 真实沙盒测试
  webhook.rs          签名与 ACK 测试
```

## 开发

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo doc --no-deps
```

Windows 下直接用包装脚本：

```powershell
.\run-vs2022.ps1 test --all-targets
```

## 故障排查

| 现象 | 原因 / 处理 |
| --- | --- |
| `vswhere` 仍显示 VS 2019 | 使用 `.\run-vs2022.ps1 ...`，脚本显式使用 VS 2022 |
| `LNK1104` 无法打开 `lnk*.tmp` | `%TEMP%` 不可写；包装脚本会把 `TMP/TEMP` 指向项目内 `.tmp` |
| `cargo` 报找不到 `link.exe` / `cl.exe` | 没有加载 MSVC 环境；使用包装脚本 |
| `401` / `11244` | AppID/Secret 错误或 Token 失效；检查凭证并重新获取 |
| `40054004` | 用户尚未添加机器人好友，C2C 主动消息不可用 |
| `40034100` | 主动消息超过频控；等待额度恢复，或改用 `msg_id` 被动回复 |
| `40034128` | 被动回复超时或次数超限；群消息 5 分钟/最多 5 次，C2C 60 分钟/最多 4 次 |
| `11253` | 当前机器人没有该接口白名单权限，需在开放平台申请 |
| 群聊收不到事件 | 确认机器人已加入该沙盒群，且消息中 @了机器人；订阅 `GROUP_MESSAGES` |
| 私聊收不到事件 | 确认用户已添加机器人好友，并使用 `C2C_MESSAGE_CREATE` 场景 |

## 许可证

Apache License 2.0，与上游 BotGo 一致。详见 [LICENSE](LICENSE)。
