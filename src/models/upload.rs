//! Rich-media upload, share link, and streaming message models.

#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

/// Request to prepare a chunked upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadPrepareRequest {
    /// 1 image, 2 video, 3 voice, 4 file.
    pub file_type: i32,
    /// File size in bytes, encoded as a string.
    pub file_size: String,
    /// File name.
    pub file_name: String,
    /// MD5 of the whole file.
    pub md5: String,
    /// SHA-1 of the whole file.
    pub sha1: String,
    /// MD5 of the first 10002432 bytes.
    pub md5_10m: String,
}

/// A presigned upload part.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UploadPart {
    /// Part index, starting at zero.
    pub index: i32,
    /// Presigned PUT URL.
    pub presigned_url: String,
    /// Part size in bytes, encoded as a string.
    pub block_size: String,
}

/// Upload behavior configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UploadConfig {
    /// Upload concurrency.
    pub concurrency: i32,
    /// Retry timeout in seconds.
    pub retry_timeout: i32,
    /// Retry delay in seconds.
    pub retry_delay: i32,
}

/// Response from preparing a chunked upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UploadPrepareResponse {
    /// Upload task ID.
    pub upload_id: String,
    /// Default part size in bytes, encoded as a string.
    pub block_size: String,
    /// Presigned parts.
    pub parts: Vec<UploadPart>,
    /// Upload behavior configuration.
    pub upload_config: UploadConfig,
}

/// Notification that one upload part completed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadPartFinish {
    /// Upload task ID.
    pub upload_id: String,
    /// Part index.
    pub part_index: i32,
    /// Actual part size in bytes, encoded as a string.
    pub block_size: String,
    /// MD5 of the part.
    pub md5: String,
}

/// Upload a rich-media file by URL, binary data, or chunked upload ID.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileUploadRequest {
    /// 1 image, 2 video, 3 voice, 4 file.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub file_type: i32,
    /// HTTP(S) source URL.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// Send immediately and consume active-message quota.
    #[serde(skip_serializing_if = "is_false")]
    pub srv_send_msg: bool,
    /// Chunked upload task ID.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub upload_id: String,
    /// File name, required for chunked or binary upload.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub file_name: String,
    /// Binary file data, at most 5 MiB.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub file_data: String,
}

const fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

const fn is_false(value: &bool) -> bool {
    !*value
}

/// Response from a rich-media upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileUploadResponse {
    /// Unique file ID.
    pub file_uuid: String,
    /// Opaque file information for the `media.file_info` message field.
    pub file_info: String,
    /// Validity of `file_info` in seconds; zero means long-lived.
    pub ttl: u32,
    /// Message ID, only when `srv_send_msg` is true.
    pub id: String,
    /// Presigned download URL, when the platform returns one.
    pub raw_url: String,
}

/// Request to prepare a Markdown media image upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaImagePrepareRequest {
    /// File name including extension.
    pub file_name: String,
    /// File size in bytes.
    pub file_size: u64,
    /// Image MIME type: `image/png`, `image/jpeg`, `image/gif`, or `image/webp`.
    pub content_type: String,
    /// Lowercase hexadecimal MD5.
    pub md5: String,
    /// Lowercase hexadecimal SHA-1.
    pub sha1: String,
}

/// Response from preparing a Markdown media image upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaImagePrepareResponse {
    /// Upload task ID, valid for four hours.
    pub upload_id: String,
    /// Presigned URL for a binary `PUT`.
    pub presigned_url: String,
    /// Presigned URL lifetime in seconds.
    pub expires_in: u32,
}

/// Request to complete a Markdown media image upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaImageCompleteRequest {
    /// Upload task ID returned by [`MediaImagePrepareResponse`].
    pub upload_id: String,
    /// MD5 ETag returned by the presigned `PUT`.
    pub md5: String,
}

/// A reusable Markdown media image.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaImage {
    /// Image ID.
    pub image_id: String,
    /// `qqbot-media://` Markdown reference.
    pub url: String,
    /// Original file name.
    pub file_name: String,
    /// Short-lived preview URL.
    pub preview_url: String,
    /// Upload time as a Unix timestamp.
    pub created_at: i64,
    /// Expiry time as a Unix timestamp.
    pub expires_at: i64,
}

/// Response from completing a Markdown media image upload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaImageCompleteResponse {
    /// Image ID.
    pub image_id: String,
    /// `qqbot-media://` Markdown reference.
    pub url: String,
    /// Sliding validity window in seconds.
    pub ttl: u32,
    /// Short-lived preview URL.
    pub preview_url: String,
}

/// A page of Markdown media images.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaImagePage {
    /// Images ordered by upload time.
    pub images: Vec<MediaImage>,
    /// Cursor for the next page; empty at the end.
    pub next_cursor: String,
    /// Total number of active images.
    pub total: u64,
}

/// Request to generate a bot share link.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UrlLinkRequest {
    /// Data passed back through the add-friend event, at most 32 characters.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub callback_data: String,
}

/// Data returned by the share-link endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UrlLinkData {
    /// Generated share URL.
    pub url: String,
}

/// Response from the share-link endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UrlLinkResponse {
    /// Share-link data.
    pub data: UrlLinkData,
}

/// A chunk of a streaming C2C message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StreamMessageRequest {
    /// `append` or `replace`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub input_mode: String,
    /// 1 generating, 10 finished.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub input_state: i32,
    /// Chunk index, starting at zero.
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub index: i32,
    /// `text` or `markdown`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content_type: String,
    /// Content for this chunk.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content_raw: String,
    /// Event ID for a passive reply.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub event_id: String,
    /// Message ID for a passive reply.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub msg_id: String,
    /// Stream message ID returned by the first chunk.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stream_msg_id: String,
    /// Message sequence number for deduplication.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub msg_seq: u32,
    /// Whether this is an interaction recall message.
    #[serde(skip_serializing_if = "is_false")]
    pub is_wakeup: bool,
}

const fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

/// Extension information on a streamed message response.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageExtInfo {
    /// Referenced message index.
    pub ref_idx: String,
}

/// Response from a streaming C2C message chunk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StreamMessageResponse {
    /// Message ID, or the stream ID for the first chunk.
    pub id: String,
    /// Send time in RFC 3339 format.
    pub timestamp: String,
    /// Extension information.
    pub ext_info: Option<MessageExtInfo>,
    /// Remaining stream length in characters.
    pub remain_msg_len: i32,
}
