//! Helpers for parsing and formatting message content.

/// A simple command parsed from message text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Command {
    /// Command name.
    pub command: String,
    /// Remaining arguments.
    pub content: String,
}

/// Removes leading `@` mentions and trims surrounding ASCII and non-breaking
/// spaces.
#[must_use]
pub fn etl_input(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(start) = remaining.find("<@!") {
        output.push_str(&remaining[..start]);
        let after_marker = &remaining[start + 3..];
        let Some(end) = after_marker.find('>') else {
            output.push_str(&remaining[start..]);
            remaining = "";
            break;
        };
        let candidate = &after_marker[..end];
        if !candidate.is_empty() && candidate.bytes().all(|byte| byte.is_ascii_digit()) {
            remaining = &after_marker[end + 1..];
        } else {
            output.push_str("<@!");
            remaining = after_marker;
        }
    }
    output.push_str(remaining);
    output.trim_matches([' ', '\u{a0}']).to_owned()
}

/// Returns the mention syntax for a user.
#[must_use]
pub fn mention_user(user_id: &str) -> String {
    format!("<@{user_id}>")
}

/// Returns the mention syntax for everyone.
///
/// This is the legacy `@everyone` form. For QQ group messages use
/// [`mention_all_members`], which emits the `qqbot-at-everyone` inline tag.
#[must_use]
pub const fn mention_all() -> &'static str {
    "@everyone"
}

/// Returns the QQ group `@all members` inline tag.
///
/// The bot must be a group administrator, and each group can use this at most
/// once per day.
#[must_use]
pub const fn mention_all_members() -> &'static str {
    "<qqbot-at-everyone />"
}

/// Returns the mention syntax for a channel.
#[must_use]
pub fn mention_channel(channel_id: &str) -> String {
    format!("<#{channel_id}>")
}

/// Returns the inline syntax for a system emoji.
#[must_use]
pub fn emoji(id: u32) -> String {
    format!("<emoji:{id}>")
}

/// Parses a command in `{command} {content}` form.
#[must_use]
pub fn parse_command(input: &str) -> Command {
    let cleaned = etl_input(input);
    let mut parts = cleaned.split(' ');
    let command = parts.next().unwrap_or_default().to_owned();
    let content = parts.collect::<Vec<_>>().join(" ");
    Command { command, content }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_at_mentions() {
        assert_eq!(etl_input("<@!123> /help me"), "/help me");
        assert_eq!(etl_input("  <@!1> <@!22>  hello  "), "hello");
        assert_eq!(etl_input("<@123> keep"), "<@123> keep");
    }

    #[test]
    fn parses_a_command() {
        let command = parse_command("<@!1> echo hello world");
        assert_eq!(command.command, "echo");
        assert_eq!(command.content, "hello world");
    }

    #[test]
    fn formats_mentions() {
        assert_eq!(mention_user("42"), "<@42>");
        assert_eq!(mention_channel("7"), "<#7>");
        assert_eq!(emoji(4), "<emoji:4>");
        assert_eq!(mention_all_members(), "<qqbot-at-everyone />");
    }
}
