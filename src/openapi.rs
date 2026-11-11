//! Async client for the QQ Bot OpenAPI v1.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use md5::{Digest, Md5};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, ETAG, USER_AGENT};
use reqwest::{Method, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sha1::Sha1;

use crate::error::{ApiError, Error, Result};
use crate::models::{
    Announces, ApiMessage, ApiPermissionDemand, ApiPermissionDemandToCreate, ApiPermissions,
    AudioControl, Channel, ChannelAnnouncesToCreate, ChannelPermissions, ChannelRolesPermissions,
    ChannelValueObject, DEFAULT_COLOR, DirectMessage, DirectMessageToCreate, Emoji, Guild,
    GuildAnnouncesToCreate, GuildMembersPager, GuildPager, GuildRoleMembersPager, GuildRoles,
    HttpIdentity, HttpReady, HttpSession, Member, MemberAddRoleBody, MemberDeleteOptions, Message,
    MessageReactionPager, MessageReactionUsers, MessageSetting, MessageToCreate, MessagesPager,
    PinsMessage, Role, Schedule, ScheduleWrapper, SendType, SettingGuide, SettingGuideToCreate,
    UpdateChannelPermissions, UpdateGuildMute, UpdateGuildMuteResponse, UpdateResult, UpdateRole,
    UpdateRoleFilter, User, WebsocketAp,
};
use crate::models::{
    BatchRemoveMembers, BatchRemoveMembersResult, CreateJoinApprovalStrategy, FileUploadRequest,
    FileUploadResponse, GatewayUrl, GroupBlacklistPage, GroupBlacklistResult, GroupBlacklistUpdate,
    GroupBotState, GroupInfo, GroupJoinRequestApproval, GroupJoinRequestPage, GroupMember,
    GroupMemberList, GroupRestrictChatSetting, JoinApprovalStrategyPage,
    JoinApprovalStrategyResult, JoinApprovalWhitelistResult, JoinApprovalWhitelistUpdate,
    MediaImageCompleteRequest, MediaImageCompleteResponse, MediaImagePage,
    MediaImagePrepareRequest, MediaImagePrepareResponse, Menu, MenuResponse, MenuUpdate,
    MenuUpdateResult, Panel, PanelCreate, PanelCreateResult, PanelDetail, PanelPage,
    PanelTargetUpdate, PanelUpdate, PanelUpdateResult, SetGroupMemberMutes, StreamMessageRequest,
    StreamMessageResponse, UpdateJoinApprovalStrategy, UploadPartFinish, UploadPrepareRequest,
    UploadPrepareResponse, UrlLinkRequest, UrlLinkResponse,
};
use crate::token::TokenSource;

/// Production OpenAPI base URL.
pub const API_BASE_URL: &str = crate::constants::API_DOMAIN;
/// Sandbox OpenAPI base URL.
pub const SANDBOX_API_BASE_URL: &str = crate::constants::SANDBOX_API_DOMAIN;
/// Header carrying the server trace ID.
pub const TRACE_ID_HEADER: &str = crate::constants::TRACE_ID_HEADER;
/// Header carrying the application ID.
pub const UNION_APP_ID_HEADER: &str = crate::constants::UNION_APP_ID_HEADER;
/// Default per-request timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Per-request options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestOptions {
    /// Overrides the default URL for this request.
    pub url: Option<String>,
    /// Hides the recall tip in supported clients.
    pub hide_tip: bool,
}

impl RequestOptions {
    /// Creates empty options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the request URL.
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Hides the recall tip.
    #[must_use]
    pub const fn with_hide_tip(mut self) -> Self {
        self.hide_tip = true;
        self
    }
}

/// Builder for [`OpenApi`].
#[derive(Clone)]
pub struct OpenApiBuilder {
    app_id: String,
    token_source: Arc<dyn TokenSource>,
    sandbox: bool,
    base_url: Option<String>,
    timeout: Duration,
    client: Option<reqwest::Client>,
}

impl std::fmt::Debug for OpenApiBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenApiBuilder")
            .field("app_id", &self.app_id)
            .field("sandbox", &self.sandbox)
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl OpenApiBuilder {
    /// Creates a builder.
    #[must_use]
    pub fn new(app_id: impl Into<String>, token_source: Arc<dyn TokenSource>) -> Self {
        Self {
            app_id: app_id.into(),
            token_source,
            sandbox: false,
            base_url: None,
            timeout: DEFAULT_TIMEOUT,
            client: None,
        }
    }

    /// Selects the sandbox environment.
    #[must_use]
    pub const fn sandbox(mut self, sandbox: bool) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Overrides the API base URL.
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the per-request timeout.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Replaces the HTTP client.
    #[must_use]
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns an error when the base URL is invalid or the HTTP client cannot
    /// be constructed.
    pub fn build(self) -> Result<OpenApi> {
        if self.app_id.is_empty() {
            return Err(Error::Config("app_id must not be empty".to_owned()));
        }

        let base_url = self.base_url.unwrap_or_else(|| {
            if self.sandbox {
                SANDBOX_API_BASE_URL.to_owned()
            } else {
                API_BASE_URL.to_owned()
            }
        });
        let base_url = Url::parse(&base_url)
            .map_err(|error| Error::Config(format!("invalid API base URL: {error}")))?;

        let client = match self.client {
            Some(client) => client,
            None => reqwest::Client::builder()
                .timeout(self.timeout)
                .build()
                .map_err(|error| Error::Config(format!("failed to build HTTP client: {error}")))?,
        };

        Ok(OpenApi {
            inner: Arc::new(OpenApiInner {
                app_id: self.app_id,
                token_source: self.token_source,
                sandbox: self.sandbox,
                base_url,
                timeout: self.timeout,
                client,
                trace_id: Mutex::new(String::new()),
            }),
        })
    }
}

/// An async QQ Bot OpenAPI client.
///
/// The client is cheap to clone: clones share the HTTP connection pool, token
/// source, and last trace ID.
#[derive(Clone)]
pub struct OpenApi {
    inner: Arc<OpenApiInner>,
}

struct OpenApiInner {
    app_id: String,
    token_source: Arc<dyn TokenSource>,
    sandbox: bool,
    base_url: Url,
    timeout: Duration,
    client: reqwest::Client,
    trace_id: Mutex<String>,
}

impl std::fmt::Debug for OpenApi {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenApi")
            .field("app_id", &self.inner.app_id)
            .field("sandbox", &self.inner.sandbox)
            .field("base_url", &self.inner.base_url)
            .finish_non_exhaustive()
    }
}

impl OpenApi {
    /// Creates a production client with default options.
    ///
    /// # Errors
    ///
    /// Returns an error if the built-in HTTP client cannot be constructed.
    pub fn new(app_id: impl Into<String>, token_source: Arc<dyn TokenSource>) -> Result<Self> {
        OpenApiBuilder::new(app_id, token_source).build()
    }

    /// Creates a sandbox client with default options.
    ///
    /// # Errors
    ///
    /// Returns an error if the built-in HTTP client cannot be constructed.
    pub fn sandbox(app_id: impl Into<String>, token_source: Arc<dyn TokenSource>) -> Result<Self> {
        OpenApiBuilder::new(app_id, token_source)
            .sandbox(true)
            .build()
    }

    /// Returns the application ID.
    #[must_use]
    pub fn app_id(&self) -> &str {
        &self.inner.app_id
    }

    /// Returns `true` when the client targets the sandbox environment.
    #[must_use]
    pub fn is_sandbox(&self) -> bool {
        self.inner.sandbox
    }

    /// Returns the API base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    /// Returns the token source used by this client.
    #[must_use]
    pub fn token_source(&self) -> Arc<dyn TokenSource> {
        Arc::clone(&self.inner.token_source)
    }

    /// Returns the trace ID of the most recent response.
    #[must_use]
    pub fn trace_id(&self) -> String {
        self.inner
            .trace_id
            .lock()
            .map(|trace_id| trace_id.clone())
            .unwrap_or_default()
    }

    /// Sends an arbitrary request through the authenticated client.
    ///
    /// # Errors
    ///
    /// Returns an error for transport failures or non-successful responses.
    pub async fn transport(
        &self,
        method: Method,
        url: &str,
        body: Option<&Value>,
    ) -> Result<Vec<u8>> {
        let url = if url.starts_with("http://") || url.starts_with("https://") {
            Url::parse(url).map_err(|error| Error::Config(format!("invalid URL: {error}")))?
        } else {
            self.inner
                .base_url
                .join(url)
                .map_err(|error| Error::Config(format!("invalid URL: {error}")))?
        };
        self.execute(method, url, &[], body, None).await
    }

    fn url_for(&self, segments: &[&str]) -> Result<Url> {
        let mut url = self.inner.base_url.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|()| Error::Config("base URL cannot be a base".to_owned()))?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }
        Ok(url)
    }

    fn record_trace_id(&self, trace_id: &str) {
        if let Ok(mut current) = self.inner.trace_id.lock() {
            trace_id.clone_into(&mut current);
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute(
        &self,
        method: Method,
        url: Url,
        query: &[(String, String)],
        body: Option<&Value>,
        extra_headers: Option<&[(&str, &str)]>,
    ) -> Result<Vec<u8>> {
        let token = self.inner.token_source.token().await?;
        let authorization =
            reqwest::header::HeaderValue::from_str(&token.authorization_header())
                .map_err(|error| Error::Config(format!("invalid token header: {error}")))?;
        let app_id = reqwest::header::HeaderValue::from_str(&self.inner.app_id)
            .map_err(|error| Error::Config(format!("invalid app ID header: {error}")))?;

        let mut request = self
            .inner
            .client
            .request(method.clone(), url.clone())
            .timeout(self.inner.timeout)
            .header(AUTHORIZATION, authorization)
            .header(UNION_APP_ID_HEADER, app_id)
            .header(USER_AGENT, crate::USER_AGENT);

        if !query.is_empty() {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        if let Some(headers) = extra_headers {
            for (name, value) in headers {
                request = request.header(*name, *value);
            }
        }

        tracing::debug!(%method, %url, "sending QQ Bot API request");
        let response = request.send().await?;
        let status = response.status();
        let trace_id = response
            .headers()
            .get(TRACE_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        self.record_trace_id(&trace_id);
        let bytes = response.bytes().await?;

        tracing::debug!(
            %method,
            %url,
            %status,
            trace_id,
            response_bytes = bytes.len(),
            "received QQ Bot API response"
        );

        if !matches!(status, StatusCode::OK | StatusCode::NO_CONTENT) {
            let body = String::from_utf8_lossy(&bytes).into_owned();
            let mut error = ApiError::from_body(status, body);
            if error.trace_id.is_empty() {
                error.trace_id.clone_from(&trace_id);
            }
            if error.is_token_expired() {
                self.inner.token_source.invalidate();
            }
            return Err(Error::Api(error));
        }

        Ok(bytes.to_vec())
    }

    async fn request_json<T>(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(String, String)],
        body: Option<&Value>,
        options: Option<&RequestOptions>,
        extra_headers: Option<&[(&str, &str)]>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut query = query.to_vec();
        if options.is_some_and(|options| options.hide_tip) {
            query.push(("hidetip".to_owned(), "true".to_owned()));
        }
        let url = if let Some(url) = options.and_then(|options| options.url.as_deref()) {
            Url::parse(url).map_err(|error| Error::Config(format!("invalid URL: {error}")))?
        } else {
            self.url_for(segments)?
        };

        let bytes = self
            .execute(method, url, &query, body, extra_headers)
            .await?;
        serde_json::from_slice(&bytes).map_err(Error::from)
    }

    async fn request_empty(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(String, String)],
        body: Option<&Value>,
        options: Option<&RequestOptions>,
        extra_headers: Option<&[(&str, &str)]>,
    ) -> Result<()> {
        let mut query = query.to_vec();
        if options.is_some_and(|options| options.hide_tip) {
            query.push(("hidetip".to_owned(), "true".to_owned()));
        }
        let url = if let Some(url) = options.and_then(|options| options.url.as_deref()) {
            Url::parse(url).map_err(|error| Error::Config(format!("invalid URL: {error}")))?
        } else {
            self.url_for(segments)?
        };
        self.execute(method, url, &query, body, extra_headers)
            .await?;
        Ok(())
    }

    fn body<T: serde::Serialize>(body: &T) -> Result<Value> {
        serde_json::to_value(body).map_err(Error::from)
    }

    // ---------------------------------------------------------------------
    // Gateway
    // ---------------------------------------------------------------------

    /// Returns the gateway URL and sharding information.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn ws(&self) -> Result<WebsocketAp> {
        self.request_json(Method::GET, &["gateway", "bot"], &[], None, None, None)
            .await
    }

    // ---------------------------------------------------------------------
    // Users
    // ---------------------------------------------------------------------

    /// Returns the current bot user.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn me(&self) -> Result<User> {
        self.request_json(Method::GET, &["users", "@me"], &[], None, None, None)
            .await
    }

    /// Returns the guilds the bot has joined.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn me_guilds(&self, pager: &GuildPager) -> Result<Vec<Guild>> {
        self.request_json(
            Method::GET,
            &["users", "@me", "guilds"],
            &pager.query_params(),
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Messages
    // ---------------------------------------------------------------------

    /// Fetches a single channel message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails or the response is malformed.
    pub async fn message(&self, channel_id: &str, message_id: &str) -> Result<Message> {
        self.message_with(channel_id, message_id, &RequestOptions::default())
            .await
    }

    /// Fetches a single channel message with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails or the response is malformed.
    pub async fn message_with(
        &self,
        channel_id: &str,
        message_id: &str,
        options: &RequestOptions,
    ) -> Result<Message> {
        let value: Value = self
            .request_json(
                Method::GET,
                &["channels", channel_id, "messages", message_id],
                &[],
                None,
                Some(options),
                None,
            )
            .await?;
        if value
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| !id.is_empty())
        {
            return serde_json::from_value(value).map_err(Error::from);
        }
        if let Some(message) = value.get("message") {
            return serde_json::from_value(message.clone()).map_err(Error::from);
        }
        Err(Error::Protocol(
            "message response did not contain a message".to_owned(),
        ))
    }

    /// Lists channel messages.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn messages(&self, channel_id: &str, pager: &MessagesPager) -> Result<Vec<Message>> {
        self.messages_with(channel_id, pager, &RequestOptions::default())
            .await
    }

    /// Lists channel messages with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn messages_with(
        &self,
        channel_id: &str,
        pager: &MessagesPager,
        options: &RequestOptions,
    ) -> Result<Vec<Message>> {
        self.request_json(
            Method::GET,
            &["channels", channel_id, "messages"],
            &pager.query_params(),
            None,
            Some(options),
            None,
        )
        .await
    }

    /// Sends a channel message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_message(
        &self,
        channel_id: &str,
        message: &MessageToCreate,
    ) -> Result<Message> {
        self.post_message_with(channel_id, message, &RequestOptions::default())
            .await
    }

    /// Sends a channel message with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_message_with(
        &self,
        channel_id: &str,
        message: &MessageToCreate,
        options: &RequestOptions,
    ) -> Result<Message> {
        let body = Self::body(message)?;
        self.request_json(
            Method::POST,
            &["channels", channel_id, "messages"],
            &[],
            Some(&body),
            Some(options),
            None,
        )
        .await
    }

    /// Edits a channel message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn patch_message(
        &self,
        channel_id: &str,
        message_id: &str,
        message: &MessageToCreate,
    ) -> Result<Message> {
        self.patch_message_with(channel_id, message_id, message, &RequestOptions::default())
            .await
    }

    /// Edits a channel message with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn patch_message_with(
        &self,
        channel_id: &str,
        message_id: &str,
        message: &MessageToCreate,
        options: &RequestOptions,
    ) -> Result<Message> {
        let body = Self::body(message)?;
        self.request_json(
            Method::PATCH,
            &["channels", channel_id, "messages", message_id],
            &[],
            Some(&body),
            Some(options),
            None,
        )
        .await
    }

    /// Recalls a channel message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn retract_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.retract_message_with(channel_id, message_id, &RequestOptions::default())
            .await
    }

    /// Recalls a channel message with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn retract_message_with(
        &self,
        channel_id: &str,
        message_id: &str,
        options: &RequestOptions,
    ) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id, "messages", message_id],
            &[],
            None,
            Some(options),
            None,
        )
        .await
    }

    /// Sends a settings guidance message that mentions the given users.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_setting_guide(
        &self,
        channel_id: &str,
        at_user_ids: &[String],
    ) -> Result<Message> {
        let mut content = String::new();
        for user_id in at_user_ids {
            content.push_str("<@");
            content.push_str(user_id);
            content.push('>');
        }
        let body = Self::body(&SettingGuideToCreate {
            content,
            setting_guide: None,
        })?;
        self.request_json(
            Method::POST,
            &["channels", channel_id, "settingguide"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Sends a group message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_group_message(
        &self,
        group_id: &str,
        message: impl Into<ApiMessage>,
    ) -> Result<Message> {
        let message = message.into();
        let endpoint = match message.send_type() {
            SendType::Text => "messages",
            SendType::RichMedia => "files",
        };
        let body = Self::body(&message)?;
        self.request_json(
            Method::POST,
            &["v2", "groups", group_id, endpoint],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Sends a C2C message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_c2c_message(
        &self,
        user_id: &str,
        message: impl Into<ApiMessage>,
    ) -> Result<Message> {
        let message = message.into();
        let endpoint = match message.send_type() {
            SendType::Text => "messages",
            SendType::RichMedia => "files",
        };
        let body = Self::body(&message)?;
        self.request_json(
            Method::POST,
            &["v2", "users", user_id, endpoint],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Recalls a C2C message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn retract_c2c_message(&self, user_id: &str, message_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["v2", "users", user_id, "messages", message_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Recalls a group message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn retract_group_message(&self, group_id: &str, message_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["v2", "groups", group_id, "messages", message_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Guilds and members
    // ---------------------------------------------------------------------

    /// Returns a guild.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn guild(&self, guild_id: &str) -> Result<Guild> {
        self.request_json(Method::GET, &["guilds", guild_id], &[], None, None, None)
            .await
    }

    /// Returns a guild member.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn guild_member(&self, guild_id: &str, user_id: &str) -> Result<Member> {
        self.request_json(
            Method::GET,
            &["guilds", guild_id, "members", user_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Lists guild members.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn guild_members(
        &self,
        guild_id: &str,
        pager: &GuildMembersPager,
    ) -> Result<Vec<Member>> {
        self.request_json(
            Method::GET,
            &["guilds", guild_id, "members"],
            &pager.query_params(),
            None,
            None,
            None,
        )
        .await
    }

    /// Lists members of a role, returning the next page cursor.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn guild_role_members(
        &self,
        guild_id: &str,
        role_id: &str,
        pager: &GuildRoleMembersPager,
    ) -> Result<(Vec<Member>, String)> {
        #[derive(serde::Deserialize, Default)]
        #[serde(default)]
        struct Response {
            data: Vec<Member>,
            next: String,
        }

        let response: Response = self
            .request_json(
                Method::GET,
                &["guilds", guild_id, "roles", role_id, "members"],
                &pager.query_params(),
                None,
                None,
                None,
            )
            .await?;
        Ok((response.data, response.next))
    }

    /// Removes a member from a guild.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_guild_member(
        &self,
        guild_id: &str,
        user_id: &str,
        options: &MemberDeleteOptions,
    ) -> Result<()> {
        let body = Self::body(options)?;
        self.request_empty(
            Method::DELETE,
            &["guilds", guild_id, "members", user_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Applies an `@everyone` mute in a guild.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn guild_mute(&self, guild_id: &str, mute: &UpdateGuildMute) -> Result<()> {
        let body = Self::body(mute)?;
        self.request_empty(
            Method::PATCH,
            &["guilds", guild_id, "mute"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Mutes a single guild member.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn member_mute(
        &self,
        guild_id: &str,
        user_id: &str,
        mute: &UpdateGuildMute,
    ) -> Result<()> {
        let body = Self::body(mute)?;
        self.request_empty(
            Method::PATCH,
            &["guilds", guild_id, "members", user_id, "mute"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Mutes multiple guild members.
    ///
    /// # Errors
    ///
    /// Returns an error when no user IDs are supplied or the request fails.
    pub async fn multi_member_mute(
        &self,
        guild_id: &str,
        mute: &UpdateGuildMute,
    ) -> Result<UpdateGuildMuteResponse> {
        if mute.user_ids.is_empty() {
            return Err(Error::Config("no user IDs supplied".to_owned()));
        }
        let body = Self::body(mute)?;
        self.request_json(
            Method::PATCH,
            &["guilds", guild_id, "mute"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Channels
    // ---------------------------------------------------------------------

    /// Returns a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn channel(&self, channel_id: &str) -> Result<Channel> {
        self.request_json(
            Method::GET,
            &["channels", channel_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Lists a guild's channels.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn channels(&self, guild_id: &str) -> Result<Vec<Channel>> {
        self.request_json(
            Method::GET,
            &["guilds", guild_id, "channels"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Creates a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_channel(
        &self,
        guild_id: &str,
        channel: &ChannelValueObject,
    ) -> Result<Channel> {
        let body = Self::body(channel)?;
        self.request_json(
            Method::POST,
            &["guilds", guild_id, "channels"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Edits a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn patch_channel(
        &self,
        channel_id: &str,
        channel: &ChannelValueObject,
    ) -> Result<Channel> {
        let body = Self::body(channel)?;
        self.request_json(
            Method::PATCH,
            &["channels", channel_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Deletes a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_channel(&self, channel_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Creates a private channel.
    ///
    /// When `user_ids` is empty, the channel is visible to administrators and
    /// members. Otherwise it is restricted to administrators and the supplied
    /// users.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_private_channel(
        &self,
        guild_id: &str,
        channel: &ChannelValueObject,
        user_ids: &[String],
    ) -> Result<Channel> {
        let mut value = channel.clone();
        value.private_type = 2;
        if !user_ids.is_empty() {
            value.private_user_ids = user_ids.to_vec();
            value.private_type = 3;
        }
        self.post_channel(guild_id, &value).await
    }

    /// Lists members currently in a voice channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn list_voice_channel_members(&self, channel_id: &str) -> Result<Vec<Member>> {
        self.request_json(
            Method::GET,
            &["channels", channel_id, "voice", "members"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Channel permissions
    // ---------------------------------------------------------------------

    /// Returns a user's permissions in a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn channel_permissions(
        &self,
        channel_id: &str,
        user_id: &str,
    ) -> Result<ChannelPermissions> {
        self.request_json(
            Method::GET,
            &["channels", channel_id, "members", user_id, "permissions"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Updates a user's permissions in a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when a permission bit string is not a `u64` or the
    /// request fails.
    pub async fn put_channel_permissions(
        &self,
        channel_id: &str,
        user_id: &str,
        permissions: &UpdateChannelPermissions,
    ) -> Result<()> {
        validate_permission_bits(permissions)?;
        let body = Self::body(permissions)?;
        self.request_empty(
            Method::PUT,
            &["channels", channel_id, "members", user_id, "permissions"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Returns a role's permissions in a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn channel_roles_permissions(
        &self,
        channel_id: &str,
        role_id: &str,
    ) -> Result<ChannelRolesPermissions> {
        self.request_json(
            Method::GET,
            &["channels", channel_id, "roles", role_id, "permissions"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Updates a role's permissions in a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when a permission bit string is not a `u64` or the
    /// request fails.
    pub async fn put_channel_roles_permissions(
        &self,
        channel_id: &str,
        role_id: &str,
        permissions: &UpdateChannelPermissions,
    ) -> Result<()> {
        validate_permission_bits(permissions)?;
        let body = Self::body(permissions)?;
        self.request_empty(
            Method::PUT,
            &["channels", channel_id, "roles", role_id, "permissions"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Audio
    // ---------------------------------------------------------------------

    /// Controls audio playback in a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_audio(&self, channel_id: &str, audio: &AudioControl) -> Result<AudioControl> {
        let body = Self::body(audio)?;
        self.request_empty(
            Method::POST,
            &["channels", channel_id, "audio"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await?;
        Ok(audio.clone())
    }

    /// Moves the bot onto the microphone.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn put_mic(&self, channel_id: &str) -> Result<()> {
        self.request_empty(
            Method::PUT,
            &["channels", channel_id, "mic"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Moves the bot off the microphone.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_mic(&self, channel_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id, "mic"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Roles
    // ---------------------------------------------------------------------

    /// Lists a guild's roles.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn roles(&self, guild_id: &str) -> Result<GuildRoles> {
        self.request_json(
            Method::GET,
            &["guilds", guild_id, "roles"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Creates a role.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_role(&self, guild_id: &str, role: &Role) -> Result<UpdateResult> {
        let body = Self::body(&role_update_body(guild_id, role))?;
        self.request_json(
            Method::POST,
            &["guilds", guild_id, "roles"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Edits a role.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn patch_role(
        &self,
        guild_id: &str,
        role_id: &str,
        role: &Role,
    ) -> Result<UpdateResult> {
        let body = Self::body(&role_update_body(guild_id, role))?;
        self.request_json(
            Method::PATCH,
            &["guilds", guild_id, "roles", role_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Deletes a role.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_role(&self, guild_id: &str, role_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["guilds", guild_id, "roles", role_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Adds a role to a member.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn member_add_role(
        &self,
        guild_id: &str,
        role_id: &str,
        user_id: &str,
        body: &MemberAddRoleBody,
    ) -> Result<()> {
        let body = Self::body(body)?;
        self.request_empty(
            Method::PUT,
            &["guilds", guild_id, "members", user_id, "roles", role_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Removes a role from a member.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn member_delete_role(
        &self,
        guild_id: &str,
        role_id: &str,
        user_id: &str,
        body: &MemberAddRoleBody,
    ) -> Result<()> {
        let body = Self::body(body)?;
        self.request_empty(
            Method::DELETE,
            &["guilds", guild_id, "members", user_id, "roles", role_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Direct messages
    // ---------------------------------------------------------------------

    /// Creates a direct-message channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_direct_message(
        &self,
        message: &DirectMessageToCreate,
    ) -> Result<DirectMessage> {
        self.create_direct_message_with(message, &RequestOptions::default())
            .await
    }

    /// Creates a direct-message channel with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_direct_message_with(
        &self,
        message: &DirectMessageToCreate,
        options: &RequestOptions,
    ) -> Result<DirectMessage> {
        let body = Self::body(message)?;
        self.request_json(
            Method::POST,
            &["users", "@me", "dms"],
            &[],
            Some(&body),
            Some(options),
            None,
        )
        .await
    }

    /// Sends a direct message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_direct_message(
        &self,
        direct_message: &DirectMessage,
        message: &MessageToCreate,
    ) -> Result<Message> {
        self.post_direct_message_with(direct_message, message, &RequestOptions::default())
            .await
    }

    /// Sends a direct message with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_direct_message_with(
        &self,
        direct_message: &DirectMessage,
        message: &MessageToCreate,
        options: &RequestOptions,
    ) -> Result<Message> {
        let body = Self::body(message)?;
        self.request_json(
            Method::POST,
            &["dms", &direct_message.guild_id, "messages"],
            &[],
            Some(&body),
            Some(options),
            None,
        )
        .await
    }

    /// Recalls a direct message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn retract_direct_message(&self, guild_id: &str, message_id: &str) -> Result<()> {
        self.retract_direct_message_with(guild_id, message_id, &RequestOptions::default())
            .await
    }

    /// Recalls a direct message with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn retract_direct_message_with(
        &self,
        guild_id: &str,
        message_id: &str,
        options: &RequestOptions,
    ) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["dms", guild_id, "messages", message_id],
            &[],
            None,
            Some(options),
            None,
        )
        .await
    }

    /// Sends a direct-message settings guide.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_direct_message_setting_guide(
        &self,
        direct_message: &DirectMessage,
        jump_guild_id: &str,
    ) -> Result<Message> {
        self.post_direct_message_setting_guide_with(
            direct_message,
            jump_guild_id,
            &RequestOptions::default(),
        )
        .await
    }

    /// Sends a direct-message settings guide with request options.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn post_direct_message_setting_guide_with(
        &self,
        direct_message: &DirectMessage,
        jump_guild_id: &str,
        options: &RequestOptions,
    ) -> Result<Message> {
        let body = Self::body(&SettingGuideToCreate {
            content: String::new(),
            setting_guide: Some(SettingGuide {
                guild_id: jump_guild_id.to_owned(),
            }),
        })?;
        self.request_json(
            Method::POST,
            &["dms", &direct_message.guild_id, "settingguide"],
            &[],
            Some(&body),
            Some(options),
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Announcements
    // ---------------------------------------------------------------------

    /// Creates a channel announcement.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_channel_announce(
        &self,
        channel_id: &str,
        announcement: &ChannelAnnouncesToCreate,
    ) -> Result<Announces> {
        let body = Self::body(announcement)?;
        self.request_json(
            Method::POST,
            &["channels", channel_id, "announces"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Deletes a channel announcement by message ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_channel_announce(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id, "announces", message_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Deletes all channel announcements.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn clean_channel_announces(&self, channel_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id, "announces", "all"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Creates a guild announcement.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_guild_announce(
        &self,
        guild_id: &str,
        announcement: &GuildAnnouncesToCreate,
    ) -> Result<Announces> {
        let body = Self::body(announcement)?;
        self.request_json(
            Method::POST,
            &["guilds", guild_id, "announces"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Deletes a guild announcement by message ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_guild_announce(&self, guild_id: &str, message_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["guilds", guild_id, "announces", message_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Deletes all guild announcements.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn clean_guild_announces(&self, guild_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["guilds", guild_id, "announces", "all"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Schedules
    // ---------------------------------------------------------------------

    /// Lists schedules for a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn list_schedules(&self, channel_id: &str, since: u64) -> Result<Vec<Schedule>> {
        self.request_json(
            Method::GET,
            &["channels", channel_id, "schedules"],
            &[("since".to_owned(), since.to_string())],
            None,
            None,
            None,
        )
        .await
    }

    /// Returns a single schedule.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn get_schedule(&self, channel_id: &str, schedule_id: &str) -> Result<Schedule> {
        self.request_json(
            Method::GET,
            &["channels", channel_id, "schedules", schedule_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Creates a schedule.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_schedule(&self, channel_id: &str, schedule: &Schedule) -> Result<Schedule> {
        let body = Self::body(&ScheduleWrapper {
            schedule: Some(schedule.clone()),
        })?;
        self.request_json(
            Method::POST,
            &["channels", channel_id, "schedules"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Edits a schedule.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn modify_schedule(
        &self,
        channel_id: &str,
        schedule_id: &str,
        schedule: &Schedule,
    ) -> Result<Schedule> {
        let body = Self::body(&ScheduleWrapper {
            schedule: Some(schedule.clone()),
        })?;
        self.request_json(
            Method::PATCH,
            &["channels", channel_id, "schedules", schedule_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Deletes a schedule.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_schedule(&self, channel_id: &str, schedule_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id, "schedules", schedule_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // API permissions
    // ---------------------------------------------------------------------

    /// Lists API permissions available to a guild.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn get_api_permissions(&self, guild_id: &str) -> Result<ApiPermissions> {
        self.request_json(
            Method::GET,
            &["guilds", guild_id, "api_permission"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Creates an API permission authorization link.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn require_api_permissions(
        &self,
        guild_id: &str,
        demand: &ApiPermissionDemandToCreate,
    ) -> Result<ApiPermissionDemand> {
        let body = Self::body(demand)?;
        self.request_json(
            Method::POST,
            &["guilds", guild_id, "api_permission", "demand"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Pins
    // ---------------------------------------------------------------------

    /// Pins a message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn add_pin(&self, channel_id: &str, message_id: &str) -> Result<PinsMessage> {
        self.request_json(
            Method::PUT,
            &["channels", channel_id, "pins", message_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Unpins a message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_pin(&self, channel_id: &str, message_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id, "pins", message_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Removes all pins from a channel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn clean_pins(&self, channel_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["channels", channel_id, "pins", "all"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Returns a channel's pinned messages.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn get_pins(&self, channel_id: &str) -> Result<PinsMessage> {
        self.request_json(
            Method::GET,
            &["channels", channel_id, "pins"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Reactions
    // ---------------------------------------------------------------------

    /// Adds the bot's reaction to a message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_message_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &Emoji,
    ) -> Result<()> {
        self.request_empty(
            Method::PUT,
            &[
                "channels",
                channel_id,
                "messages",
                message_id,
                "reactions",
                &emoji.emoji_type.to_string(),
                &emoji.id,
            ],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Removes the bot's reaction from a message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_own_message_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &Emoji,
    ) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &[
                "channels",
                channel_id,
                "messages",
                message_id,
                "reactions",
                &emoji.emoji_type.to_string(),
                &emoji.id,
            ],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Lists users that reacted with an emoji.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn get_message_reaction_users(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &Emoji,
        pager: &MessageReactionPager,
    ) -> Result<MessageReactionUsers> {
        self.request_json(
            Method::GET,
            &[
                "channels",
                channel_id,
                "messages",
                message_id,
                "reactions",
                &emoji.emoji_type.to_string(),
                &emoji.id,
            ],
            &pager.query_params(),
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Interactions
    // ---------------------------------------------------------------------

    /// Updates an interaction with a raw JSON response body.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn put_interaction(&self, interaction_id: &str, body: &str) -> Result<()> {
        let value: Value = serde_json::from_str(body)
            .map_err(|error| Error::Config(format!("interaction body is not JSON: {error}")))?;
        self.request_empty(
            Method::PUT,
            &["interactions", interaction_id],
            &[],
            Some(&value),
            None,
            Some(&[("X-Callback-AppID", self.app_id())]),
        )
        .await
    }

    // ---------------------------------------------------------------------
    // HTTP gateway
    // ---------------------------------------------------------------------

    /// Creates an HTTP callback session.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_session(&self, identity: &HttpIdentity) -> Result<HttpReady> {
        let body = Self::body(identity)?;
        self.request_json(
            Method::POST,
            &["gateway", "webhook", "sessions"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Checks HTTP callback sessions, resuming inactive ones server-side.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn check_sessions(&self) -> Result<Vec<HttpSession>> {
        self.request_json(
            Method::PATCH,
            &["gateway", "webhook", "sessions"],
            &[("action".to_owned(), "check".to_owned())],
            None,
            None,
            None,
        )
        .await
    }

    /// Lists active HTTP callback sessions.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn session_list(&self) -> Result<Vec<HttpSession>> {
        self.request_json(
            Method::GET,
            &["gateway", "webhook", "sessions"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Removes an HTTP callback session.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["gateway", "webhook", "sessions", session_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Message settings
    // ---------------------------------------------------------------------

    /// Returns message frequency settings for a guild.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn get_message_setting(&self, guild_id: &str) -> Result<MessageSetting> {
        self.request_json(
            Method::GET,
            &["guilds", guild_id, "message", "setting"],
            &[],
            None,
            None,
            None,
        )
        .await
    }
}

fn validate_permission_bits(permissions: &UpdateChannelPermissions) -> Result<()> {
    if !permissions.add.is_empty() {
        permissions
            .add
            .parse::<u64>()
            .map_err(|error| Error::Config(format!("invalid permission add value: {error}")))?;
    }
    if !permissions.remove.is_empty() {
        permissions
            .remove
            .parse::<u64>()
            .map_err(|error| Error::Config(format!("invalid permission remove value: {error}")))?;
    }
    Ok(())
}

fn role_update_body(guild_id: &str, role: &Role) -> UpdateRole {
    let mut role = role.clone();
    if role.color == 0 {
        role.color = DEFAULT_COLOR;
    }
    UpdateRole {
        guild_id: guild_id.to_owned(),
        filter: UpdateRoleFilter {
            name: 1,
            color: 1,
            hoist: 1,
        },
        info: role,
    }
}

fn paging_query(cursor: Option<&str>, limit: Option<u32>) -> Vec<(String, String)> {
    let mut query = Vec::new();
    if let Some(cursor) = cursor.filter(|cursor| !cursor.is_empty()) {
        query.push(("cursor".to_owned(), cursor.to_owned()));
    }
    if let Some(limit) = limit {
        query.push(("limit".to_owned(), limit.to_string()));
    }
    query
}

/// QQ group management, menu, panel, upload, and streaming APIs.
impl OpenApi {
    /// Returns the generic WebSocket gateway URL.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn gateway(&self) -> Result<GatewayUrl> {
        self.request_json(Method::GET, &["gateway"], &[], None, None, None)
            .await
    }

    /// Returns basic information about a group.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn group_info(&self, group_openid: &str) -> Result<GroupInfo> {
        self.request_json(
            Method::GET,
            &["v2", "groups", group_openid, "info"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Returns the bot's state in a group.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn group_bot_state(&self, group_openid: &str) -> Result<GroupBotState> {
        self.request_json(
            Method::GET,
            &["v2", "groups", group_openid, "bot_state"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Lists group members.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn group_members(
        &self,
        group_openid: &str,
        cursor: Option<&str>,
    ) -> Result<GroupMemberList> {
        self.request_json(
            Method::GET,
            &["v2", "groups", group_openid, "members"],
            &paging_query(cursor, None),
            None,
            None,
            None,
        )
        .await
    }

    /// Returns one group member.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn group_member(
        &self,
        group_openid: &str,
        member_openid: &str,
    ) -> Result<GroupMember> {
        self.request_json(
            Method::GET,
            &["v2", "groups", group_openid, "members", member_openid],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Lists group blacklist entries.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn group_blacklist(
        &self,
        group_openid: &str,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<GroupBlacklistPage> {
        self.request_json(
            Method::GET,
            &["v2", "groups", group_openid, "member_blacklist"],
            &paging_query(cursor, limit),
            None,
            None,
            None,
        )
        .await
    }

    /// Adds or removes group blacklist entries.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn update_group_blacklist(
        &self,
        group_openid: &str,
        update: &GroupBlacklistUpdate,
    ) -> Result<GroupBlacklistResult> {
        let body = Self::body(update)?;
        self.request_json(
            Method::POST,
            &["v2", "groups", group_openid, "member_blacklist"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Removes up to 20 group members in one request.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn batch_remove_group_members(
        &self,
        group_openid: &str,
        request: &BatchRemoveMembers,
    ) -> Result<BatchRemoveMembersResult> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "groups", group_openid, "batch_remove_members"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Lists pending group join requests.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn group_join_requests(
        &self,
        group_openid: &str,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<GroupJoinRequestPage> {
        self.request_json(
            Method::GET,
            &["v2", "groups", group_openid, "join_request_list"],
            &paging_query(cursor, limit),
            None,
            None,
            None,
        )
        .await
    }

    /// Approves or rejects a group join request.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn approve_group_join_request(
        &self,
        group_openid: &str,
        member_openid: &str,
        approval: &GroupJoinRequestApproval,
    ) -> Result<()> {
        let body = Self::body(approval)?;
        self.request_empty(
            Method::POST,
            &[
                "v2",
                "groups",
                group_openid,
                "approval_join_request",
                member_openid,
            ],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Returns the current group mute configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn group_restrict_chat_setting(
        &self,
        group_openid: &str,
    ) -> Result<GroupRestrictChatSetting> {
        self.request_json(
            Method::GET,
            &["v2", "groups", group_openid, "restrict_chat_setting"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Sets member-level mutes in a group.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn set_group_member_mutes(
        &self,
        group_openid: &str,
        request: &SetGroupMemberMutes,
    ) -> Result<()> {
        let body = Self::body(request)?;
        self.request_empty(
            Method::POST,
            &["v2", "groups", group_openid, "restrict_chat_setting"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Lists automatic join approval strategies.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn join_approval_strategies(
        &self,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<JoinApprovalStrategyPage> {
        self.request_json(
            Method::GET,
            &["v2", "groups", "join_approval_strategy"],
            &paging_query(cursor, limit),
            None,
            None,
            None,
        )
        .await
    }

    /// Creates an automatic join approval strategy.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_join_approval_strategy(
        &self,
        request: &CreateJoinApprovalStrategy,
    ) -> Result<JoinApprovalStrategyResult> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "groups", "join_approval_strategy"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Deletes an automatic join approval strategy.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_join_approval_strategy(&self, strategy_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["v2", "groups", "join_approval_strategy", strategy_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Updates an automatic join approval strategy.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn update_join_approval_strategy(
        &self,
        strategy_id: &str,
        request: &UpdateJoinApprovalStrategy,
    ) -> Result<JoinApprovalStrategyResult> {
        let body = Self::body(request)?;
        self.request_json(
            Method::PATCH,
            &["v2", "groups", "join_approval_strategy", strategy_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Executes a full scan for an automatic join approval strategy.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn execute_join_approval_strategy(&self, strategy_id: &str) -> Result<()> {
        self.request_empty(
            Method::POST,
            &[
                "v2",
                "groups",
                "join_approval_strategy",
                strategy_id,
                "execute",
            ],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Adds or removes numbers from a strategy whitelist.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn update_join_approval_whitelist(
        &self,
        strategy_id: &str,
        request: &JoinApprovalWhitelistUpdate,
    ) -> Result<JoinApprovalWhitelistResult> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &[
                "v2",
                "groups",
                "join_approval_strategy",
                strategy_id,
                "whitelist_users",
            ],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Returns the global C2C custom menu.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn menu(&self) -> Result<MenuResponse> {
        self.request_json(Method::GET, &["v2", "menu"], &[], None, None, None)
            .await
    }

    /// Replaces the global C2C custom menu.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn update_menu(&self, menu: &Menu) -> Result<MenuUpdateResult> {
        let body = Self::body(&MenuUpdate { menu: menu.clone() })?;
        self.request_json(Method::PUT, &["v2", "menu"], &[], Some(&body), None, None)
            .await
    }

    /// Lists command panels for a scope.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn panels(
        &self,
        scope: &str,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> Result<PanelPage> {
        let mut query = vec![("scope".to_owned(), scope.to_owned())];
        query.extend(paging_query(cursor, limit));
        self.request_json(Method::GET, &["v2", "panels"], &query, None, None, None)
            .await
    }

    /// Creates a command panel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn create_panel(&self, request: &PanelCreate) -> Result<PanelCreateResult> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "panels"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Returns a command panel's full configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn panel(&self, panel_id: &str) -> Result<PanelDetail> {
        self.request_json(
            Method::GET,
            &["v2", "panels", panel_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Replaces a panel's items and note.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn update_panel(&self, panel_id: &str, panel: &Panel) -> Result<PanelUpdateResult> {
        let body = Self::body(&PanelUpdate {
            panel: panel.clone(),
        })?;
        self.request_json(
            Method::PUT,
            &["v2", "panels", panel_id],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Deletes a command panel.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_panel(&self, panel_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["v2", "panels", panel_id],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Adds or removes panel targets.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn update_panel_targets(
        &self,
        panel_id: &str,
        request: &PanelTargetUpdate,
    ) -> Result<()> {
        let body = Self::body(request)?;
        self.request_empty(
            Method::PUT,
            &["v2", "panels", panel_id, "target"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Generates a bot share link.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn generate_url_link(&self, request: &UrlLinkRequest) -> Result<UrlLinkResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "generate_url_link"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Uploads or registers rich media for a group.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn upload_group_file(
        &self,
        group_openid: &str,
        request: &FileUploadRequest,
    ) -> Result<FileUploadResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "groups", group_openid, "files"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Uploads or registers rich media for a C2C chat.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn upload_user_file(
        &self,
        user_openid: &str,
        request: &FileUploadRequest,
    ) -> Result<FileUploadResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "users", user_openid, "files"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Prepares a chunked rich-media upload for a group.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn prepare_group_upload(
        &self,
        group_openid: &str,
        request: &UploadPrepareRequest,
    ) -> Result<UploadPrepareResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "groups", group_openid, "upload_prepare"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Marks one group upload part as complete.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn finish_group_upload_part(
        &self,
        group_openid: &str,
        request: &UploadPartFinish,
    ) -> Result<()> {
        let body = Self::body(request)?;
        self.request_empty(
            Method::POST,
            &["v2", "groups", group_openid, "upload_part_finish"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Prepares a chunked rich-media upload for a C2C chat.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn prepare_user_upload(
        &self,
        user_openid: &str,
        request: &UploadPrepareRequest,
    ) -> Result<UploadPrepareResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "users", user_openid, "upload_prepare"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Marks one C2C upload part as complete.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn finish_user_upload_part(
        &self,
        user_openid: &str,
        request: &UploadPartFinish,
    ) -> Result<()> {
        let body = Self::body(request)?;
        self.request_empty(
            Method::POST,
            &["v2", "users", user_openid, "upload_part_finish"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Prepares a Markdown media image upload.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn prepare_media_image(
        &self,
        request: &MediaImagePrepareRequest,
    ) -> Result<MediaImagePrepareResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "media-images", "prepare"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Completes a Markdown media image upload after the presigned `PUT`.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn complete_media_image(
        &self,
        request: &MediaImageCompleteRequest,
    ) -> Result<MediaImageCompleteResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "media-images", "complete"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }

    /// Lists reusable Markdown media images.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn media_images(&self, cursor: Option<&str>) -> Result<MediaImagePage> {
        self.request_json(
            Method::GET,
            &["v2", "media-images"],
            &paging_query(cursor, None),
            None,
            None,
            None,
        )
        .await
    }

    /// Deletes a reusable Markdown media image.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn delete_media_image(&self, image_id: &str) -> Result<()> {
        self.request_empty(
            Method::DELETE,
            &["v2", "media-images", image_id, "delete"],
            &[],
            None,
            None,
            None,
        )
        .await
    }

    /// Uploads an image for use in Markdown and returns its reusable reference.
    ///
    /// The image is limited to 10 MiB. The method computes MD5 and SHA-1,
    /// performs the presigned binary `PUT`, and completes the upload using the
    /// returned ETag.
    ///
    /// # Errors
    ///
    /// Returns an error for oversized files, failed uploads, or invalid
    /// responses.
    pub async fn upload_media_image(
        &self,
        file_name: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<MediaImageCompleteResponse> {
        const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(Error::Config(format!(
                "media image is too large: {} bytes",
                bytes.len()
            )));
        }

        let mut md5 = Md5::new();
        md5.update(bytes);
        let md5_hex = hex::encode(md5.finalize());

        let mut sha1 = Sha1::new();
        sha1.update(bytes);
        let sha1_hex = hex::encode(sha1.finalize());

        let prepared = self
            .prepare_media_image(&MediaImagePrepareRequest {
                file_name: file_name.to_owned(),
                file_size: u64::try_from(bytes.len())
                    .map_err(|_| Error::Config("image length does not fit in u64".to_owned()))?,
                content_type: content_type.to_owned(),
                md5: md5_hex.clone(),
                sha1: sha1_hex,
            })
            .await?;

        let response = self
            .inner
            .client
            .put(&prepared.presigned_url)
            .timeout(self.inner.timeout)
            .header(CONTENT_TYPE, content_type)
            .body(bytes.to_vec())
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await?;
            return Err(Error::Protocol(format!(
                "media image upload failed: HTTP {status}: {body}"
            )));
        }

        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.trim_matches('"').to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or(md5_hex);

        self.complete_media_image(&MediaImageCompleteRequest {
            upload_id: prepared.upload_id,
            md5: etag,
        })
        .await
    }

    /// Sends one chunk of a streaming C2C message.
    ///
    /// # Errors
    ///
    /// Returns an error when the request fails.
    pub async fn stream_c2c_message(
        &self,
        user_openid: &str,
        request: &StreamMessageRequest,
    ) -> Result<StreamMessageResponse> {
        let body = Self::body(request)?;
        self.request_json(
            Method::POST,
            &["v2", "users", user_openid, "stream_messages"],
            &[],
            Some(&body),
            None,
            None,
        )
        .await
    }
}
