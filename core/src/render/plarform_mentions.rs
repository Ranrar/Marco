//! Platform mention mapping and URL generation.
//!
//! This module contains the supported platform list and logic for turning a
//! `(platform, username)` pair into a profile URL.

/// Build an external profile URL for a supported platform.
///
/// Returns `None` if the platform is unknown.
pub fn profile_url(platform: &str, username: &str) -> Option<String> {
    let p = platform.trim().to_ascii_lowercase();
    let username = username.trim();
    let u = encode_path_segment(username);

    if u.is_empty() {
        return None;
    }

    match p.as_str() {
        // Developer / code
        "github" => Some(format!("https://github.com/{u}")),
        "gitlab" => Some(format!("https://gitlab.com/{u}")),
        "bitbucket" => Some(format!("https://bitbucket.org/{u}")),
        "codeberg" => Some(format!("https://codeberg.org/{u}")),

        // Social
        "twitter" => Some(format!("https://twitter.com/{u}")),
        "x" => Some(format!("https://x.com/{u}")),
        "reddit" => Some(format!("https://www.reddit.com/user/{u}")),
        "instagram" => Some(format!("https://www.instagram.com/{u}/")),
        "snapchat" => Some(format!("https://www.snapchat.com/@{u}")),
        "tiktok" => Some(format!("https://www.tiktok.com/@{u}")),
        "youtube" => Some(format!("https://www.youtube.com/@{u}")),
        "linkedin" => Some(format!("https://www.linkedin.com/in/{u}")),
        "xing" => Some(format!("https://www.xing.com/profile/{u}")),
        "facebook" => Some(format!("https://www.facebook.com/{u}")),
        "threads" => Some(format!("https://www.threads.net/@{u}")),
        "twitch" => Some(format!("https://www.twitch.tv/{u}")),
        "soundcloud" => Some(format!("https://soundcloud.com/{u}")),
        "mixcloud" => Some(format!("https://www.mixcloud.com/{u}/")),
        "telegram" => Some(format!("https://t.me/{u}")),
        "vk" | "vkontakte" => Some(format!("https://vk.com/{u}")),

        // Social / discovery / publishing
        "pinterest" => Some(format!("https://www.pinterest.com/{u}/")),
        "medium" => Some(format!("https://medium.com/@{u}")),
        "tumblr" => Some(format!("https://www.tumblr.com/{u}")),
        "quora" => Some(format!("https://www.quora.com/profile/{u}")),
        "myspace" => Some(format!("https://myspace.com/{u}")),
        "dribbble" => Some(format!("https://dribbble.com/{u}")),
        "9gag" => Some(format!("https://9gag.com/u/{u}")),
        "bluesky" | "bsky" => Some(format!("https://bsky.app/profile/{u}")),
        "likee" => Some(format!("https://www.likee.video/@{u}")),

        // Regional / specialized
        "zhihu" => Some(format!("https://www.zhihu.com/people/{u}")),
        "bilibili" => Some(format!("https://space.bilibili.com/{u}")),
        "tieba" | "baidutieba" | "baidu-tieba" | "baidu_tieba" => {
            let q = encode_query_component(username);
            Some(format!("https://tieba.baidu.com/home/main/?un={q}"))
        }

        // Fediverse defaults
        // NOTE: These platforms are instance-based; without an instance in the
        // syntax, we pick a reasonable default instance.
        "mastodon" => Some(format!("https://mastodon.social/@{u}")),
        "pixelfed" => Some(format!("https://pixelfed.social/{u}")),

        // Chat/community
        "discord" => Some(format!("https://discord.com/users/{u}")),

        _ => None,
    }
}

/// Percent-encode a string for safe embedding as a single URL path segment.
///
/// This intentionally uses a simple "unreserved" set (RFC 3986):
/// ALPHA / DIGIT / "-" / "." / "_" / "~".
fn encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());

    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*b as char)
            }
            other => {
                out.push('%');
                out.push(HEX[(other >> 4) as usize] as char);
                out.push(HEX[(other & 0x0f) as usize] as char);
            }
        }
    }

    out
}

/// Percent-encode a string for safe embedding as a query component.
///
/// This intentionally uses the same escaping as `encode_path_segment()`.
fn encode_query_component(s: &str) -> String {
    encode_path_segment(s)
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_profile_url_github() {
        assert_eq!(
            profile_url("github", "ranrar").as_deref(),
            Some("https://github.com/ranrar")
        );
    }

    #[test]
    fn smoke_test_profile_url_xing() {
        assert_eq!(
            profile_url("xing", "John_Doe").as_deref(),
            Some("https://www.xing.com/profile/John_Doe")
        );
    }

    #[test]
    fn smoke_test_profile_url_vk_alias() {
        assert_eq!(
            profile_url("vkontakte", "durov").as_deref(),
            Some("https://vk.com/durov")
        );
    }

    #[test]
    fn smoke_test_profile_url_unknown_platform() {
        assert!(profile_url("unknown", "ranrar").is_none());
    }

    #[test]
    fn smoke_test_profile_url_pinterest() {
        assert_eq!(
            profile_url("pinterest", "Pinterest").as_deref(),
            Some("https://www.pinterest.com/Pinterest/")
        );
    }

    #[test]
    fn smoke_test_profile_url_medium() {
        assert_eq!(
            profile_url("medium", "rapidseedbox").as_deref(),
            Some("https://medium.com/@rapidseedbox")
        );
    }

    #[test]
    fn smoke_test_profile_url_bluesky_handle_with_dot() {
        assert_eq!(
            profile_url("bluesky", "bsky.app").as_deref(),
            Some("https://bsky.app/profile/bsky.app")
        );
    }

    #[test]
    fn smoke_test_profile_url_snapchat() {
        assert_eq!(
            profile_url("snapchat", "teamsnapchat").as_deref(),
            Some("https://www.snapchat.com/@teamsnapchat")
        );
    }

    #[test]
    fn smoke_test_profile_url_likee() {
        assert_eq!(
            profile_url("likee", "likee").as_deref(),
            Some("https://www.likee.video/@likee")
        );
    }

    #[test]
    fn smoke_test_profile_url_tieba_query_param() {
        assert_eq!(
            profile_url("tieba", "\u{8d34}\u{5427}\u{5b98}\u{65b9}").as_deref(),
            Some("https://tieba.baidu.com/home/main/?un=%E8%B4%B4%E5%90%A7%E5%AE%98%E6%96%B9")
        );
    }

    #[test]
    fn smoke_test_encode_path_segment_encodes_reserved() {
        assert_eq!(encode_path_segment("a b"), "a%20b");
        assert_eq!(encode_path_segment("a/b"), "a%2Fb");
        assert_eq!(encode_path_segment("a?b"), "a%3Fb");
    }
}
