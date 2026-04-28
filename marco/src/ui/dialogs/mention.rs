//! Insert Mention Dialog
//!
//! Provides a platform-aware dialog for inserting Marco platform mentions.

use gio::prelude::*;
use gtk4::{
    gdk, glib, prelude::*, Align, Box, Button, Entry, Label, Orientation, Picture, PolicyType,
    ScrolledWindow, Window,
};
use rsvg::{CairoRenderer, Loader};
use sourceview5::{Buffer, View};
use std::{cell::RefCell, rc::Rc, time::Duration};

const MAX_USERNAME_LEN: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    Username,
    NumericId,
    DomainHandle,
}

#[derive(Debug, Clone, Copy)]
struct MentionInputProfile {
    label: &'static str,
    placeholder: &'static str,
    helper: &'static str,
    kind: InputKind,
}

#[derive(Debug, Clone)]
struct ProfileCheckResult {
    token: u64,
    exists: Option<bool>,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationCapability {
    Supported,
    Unsupported,
    NotImplemented,
}

fn validation_capability(platform_key: &str) -> ValidationCapability {
    if matches!(
        platform_key,
        "x" | "twitter"
            | "instagram"
            | "threads"
            | "facebook"
            | "linkedin"
            | "tiktok"
            | "snapchat"
            | "discord"
    ) {
        ValidationCapability::Unsupported
    } else if matches!(platform_key, "github" | "gitlab" | "reddit" | "mastodon") {
        ValidationCapability::Supported
    } else {
        ValidationCapability::NotImplemented
    }
}

fn validation_helper_note(platform_key: &str) -> &'static str {
    match validation_capability(platform_key) {
        ValidationCapability::Supported => {
            "Validation: supported on this platform (live profile check available)."
        }
        ValidationCapability::Unsupported => {
            "Validation: this platform does not support reliable public profile validation."
        }
        ValidationCapability::NotImplemented => {
            "Validation: not implemented for this platform yet."
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidButtonState {
    Neutral,
    Valid,
    Error,
}

fn set_valid_button_state(valid_button: &Button, state: ValidButtonState, tooltip: &str) {
    valid_button.remove_css_class("marco-btn-green");
    valid_button.remove_css_class("marco-btn-red");
    valid_button.set_tooltip_text(Some(tooltip));

    match state {
        ValidButtonState::Neutral => {
            valid_button.set_label("Valid");
            valid_button.set_sensitive(false);
        }
        ValidButtonState::Valid => {
            valid_button.set_label("Valid");
            valid_button.add_css_class("marco-btn-green");
            valid_button.set_sensitive(true);
        }
        ValidButtonState::Error => {
            valid_button.set_label("Error");
            valid_button.add_css_class("marco-btn-red");
            valid_button.set_sensitive(true);
        }
    }
}

fn is_domain_like(value: &str) -> bool {
    if !value.contains('.') {
        return false;
    }

    let labels: Vec<&str> = value.split('.').collect();
    if labels.len() < 2 {
        return false;
    }

    labels.iter().all(|label| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

fn is_handle_like(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn is_bot_heavy_platform(platform_key: &str) -> bool {
    matches!(
        platform_key,
        "x" | "twitter" | "instagram" | "threads" | "facebook" | "linkedin" | "tiktok" | "snapchat"
    )
}

fn classify_http_status(platform_key: &str, status: reqwest::StatusCode) -> ProfileCheckResult {
    let code = status.as_u16();
    if (200..300).contains(&code) {
        if is_bot_heavy_platform(platform_key) {
            ProfileCheckResult {
                token: 0,
                exists: None,
                message: "Page reachable, but this site may block bot-style validation."
                    .to_string(),
            }
        } else {
            ProfileCheckResult {
                token: 0,
                exists: Some(true),
                message: "Profile found.".to_string(),
            }
        }
    } else if code == 404 || code == 410 {
        ProfileCheckResult {
            token: 0,
            exists: Some(false),
            message: "Profile not found.".to_string(),
        }
    } else if code == 401 || code == 403 {
        ProfileCheckResult {
            token: 0,
            exists: None,
            message: "Profile check blocked (private, restricted, or anti-bot).".to_string(),
        }
    } else if code == 429 {
        ProfileCheckResult {
            token: 0,
            exists: None,
            message: "Rate limited while checking profile.".to_string(),
        }
    } else {
        ProfileCheckResult {
            token: 0,
            exists: None,
            message: format!("Could not confirm profile (HTTP {}).", code),
        }
    }
}

fn request_status_with_head_fallback(
    client: &reqwest::blocking::Client,
    url: &str,
) -> Result<reqwest::StatusCode, reqwest::Error> {
    match client.head(url).send() {
        Ok(resp) => {
            let status = resp.status();
            if matches!(status.as_u16(), 401 | 403 | 405) {
                client.get(url).send().map(|r| r.status())
            } else {
                Ok(status)
            }
        }
        Err(_) => client.get(url).send().map(|r| r.status()),
    }
}

fn check_via_api(
    client: &reqwest::blocking::Client,
    platform_key: &str,
    identifier: &str,
) -> Option<ProfileCheckResult> {
    match platform_key {
        "github" => {
            let url = format!("https://api.github.com/users/{identifier}");
            let status = client.get(url).send().ok()?.status();
            Some(classify_http_status(platform_key, status))
        }
        "gitlab" => {
            let url = format!("https://gitlab.com/api/v4/users?username={identifier}");
            let resp = client.get(url).send().ok()?;
            let status = resp.status();
            if !status.is_success() {
                return Some(classify_http_status(platform_key, status));
            }

            let body = resp.text().ok()?;
            let parsed = serde_json::from_str::<serde_json::Value>(&body).ok();
            match parsed {
                Some(serde_json::Value::Array(items)) => {
                    if items.iter().any(|item| {
                        item.get("username")
                            .and_then(|u| u.as_str())
                            .map(|u| u.eq_ignore_ascii_case(identifier))
                            .unwrap_or(false)
                    }) {
                        Some(ProfileCheckResult {
                            token: 0,
                            exists: Some(true),
                            message: "Profile found via GitLab API.".to_string(),
                        })
                    } else {
                        Some(ProfileCheckResult {
                            token: 0,
                            exists: Some(false),
                            message: "Profile not found via GitLab API.".to_string(),
                        })
                    }
                }
                _ => Some(ProfileCheckResult {
                    token: 0,
                    exists: None,
                    message: "Could not parse GitLab API response.".to_string(),
                }),
            }
        }
        "reddit" => {
            let url = format!("https://www.reddit.com/user/{identifier}/about.json");
            let status = client.get(url).send().ok()?.status();
            Some(classify_http_status(platform_key, status))
        }
        "mastodon" => {
            let url = format!("https://mastodon.social/api/v1/accounts/lookup?acct={identifier}");
            let status = client.get(url).send().ok()?.status();
            Some(classify_http_status(platform_key, status))
        }
        _ => None,
    }
}

fn check_profile_exists(platform_key: &str, identifier: &str) -> ProfileCheckResult {
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("text/html,application/json;q=0.9,*/*;q=0.8"),
    );

    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .redirect(reqwest::redirect::Policy::limited(4))
        .user_agent("Marco Mention Validator/1.0")
        .default_headers(default_headers)
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return ProfileCheckResult {
                token: 0,
                exists: None,
                message: "Could not initialize live profile validator.".to_string(),
            }
        }
    };

    if let Some(api_result) = check_via_api(&client, platform_key, identifier) {
        return api_result;
    }

    let Some(url) = marco_core::render::plarform_mentions::profile_url(platform_key, identifier)
    else {
        return ProfileCheckResult {
            token: 0,
            exists: None,
            message: "Unsupported platform for profile validation.".to_string(),
        };
    };

    match request_status_with_head_fallback(&client, &url) {
        Ok(status) => classify_http_status(platform_key, status),
        Err(err) => {
            let message = if err.is_timeout() {
                "Profile check timed out.".to_string()
            } else if err.is_connect() {
                "Could not connect to profile host.".to_string()
            } else {
                "Could not verify profile right now.".to_string()
            };

            ProfileCheckResult {
                token: 0,
                exists: None,
                message,
            }
        }
    }
}

fn profile_for_platform(platform_key: &str) -> MentionInputProfile {
    match platform_key {
        "github" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your GitHub username (the value after github.com/).",
            kind: InputKind::Username,
        },
        "gitlab" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your GitLab username (the value after gitlab.com/).",
            kind: InputKind::Username,
        },
        "bitbucket" => MentionInputProfile {
            label: "Workspace/User",
            placeholder: "e.g. ranrar",
            helper: "Use your Bitbucket profile/workspace name.",
            kind: InputKind::Username,
        },
        "codeberg" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Codeberg username.",
            kind: InputKind::Username,
        },
        "x" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your X handle without the @.",
            kind: InputKind::Username,
        },
        "twitter" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your Twitter handle without the @.",
            kind: InputKind::Username,
        },
        "reddit" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Reddit username (without u/).",
            kind: InputKind::Username,
        },
        "instagram" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your Instagram handle without the @.",
            kind: InputKind::Username,
        },
        "snapchat" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your Snapchat public username/handle.",
            kind: InputKind::Username,
        },
        "tiktok" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your TikTok handle without the @.",
            kind: InputKind::Username,
        },
        "youtube" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your YouTube handle without the @.",
            kind: InputKind::Username,
        },
        "linkedin" => MentionInputProfile {
            label: "Profile Slug",
            placeholder: "e.g. ranrar",
            helper: "Use the LinkedIn profile slug from /in/<slug>.",
            kind: InputKind::Username,
        },
        "xing" => MentionInputProfile {
            label: "Profile Slug",
            placeholder: "e.g. ranrar",
            helper: "Use the XING profile slug after /profile/.",
            kind: InputKind::Username,
        },
        "facebook" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use the Facebook profile username/vanity name.",
            kind: InputKind::Username,
        },
        "threads" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your Threads handle without the @.",
            kind: InputKind::Username,
        },
        "twitch" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Twitch channel username.",
            kind: InputKind::Username,
        },
        "soundcloud" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your SoundCloud profile name from the URL.",
            kind: InputKind::Username,
        },
        "mixcloud" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Mixcloud username from the profile URL.",
            kind: InputKind::Username,
        },
        "telegram" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Telegram username used in t.me links.",
            kind: InputKind::Username,
        },
        "vk" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your VK short username.",
            kind: InputKind::Username,
        },
        "pinterest" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Pinterest username from the profile URL.",
            kind: InputKind::Username,
        },
        "medium" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Medium username (the value after @ in profile URLs).",
            kind: InputKind::Username,
        },
        "tumblr" => MentionInputProfile {
            label: "Blog Name",
            placeholder: "e.g. ranrar",
            helper: "Use your Tumblr blog/profile name.",
            kind: InputKind::Username,
        },
        "quora" => MentionInputProfile {
            label: "Profile Slug",
            placeholder: "e.g. ranrar",
            helper: "Use your Quora profile slug after /profile/.",
            kind: InputKind::Username,
        },
        "myspace" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Myspace profile name.",
            kind: InputKind::Username,
        },
        "dribbble" => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use your Dribbble username.",
            kind: InputKind::Username,
        },
        "9gag" => MentionInputProfile {
            label: "User Name",
            placeholder: "e.g. ranrar",
            helper: "Use your 9GAG user name from /u/<name>.",
            kind: InputKind::Username,
        },
        "bluesky" => MentionInputProfile {
            label: "Handle / Domain",
            placeholder: "e.g. bsky.app",
            helper: "Use your Bluesky handle/domain (for example name.bsky.social).",
            kind: InputKind::DomainHandle,
        },
        "likee" => MentionInputProfile {
            label: "Handle",
            placeholder: "e.g. ranrar",
            helper: "Use your Likee public handle without the @.",
            kind: InputKind::Username,
        },
        "zhihu" => MentionInputProfile {
            label: "Profile Name",
            placeholder: "e.g. ranrar",
            helper: "Use your Zhihu profile name after /people/.",
            kind: InputKind::Username,
        },
        "bilibili" => MentionInputProfile {
            label: "User ID",
            placeholder: "e.g. 2",
            helper: "Use the numeric Bilibili UID.",
            kind: InputKind::NumericId,
        },
        "tieba" => MentionInputProfile {
            label: "Baidu Name",
            placeholder: "e.g. ranrar",
            helper: "Use your Baidu Tieba username.",
            kind: InputKind::Username,
        },
        "mastodon" => MentionInputProfile {
            label: "Account Name",
            placeholder: "e.g. ranrar",
            helper: "Use account name only (default instance is mastodon.social).",
            kind: InputKind::Username,
        },
        "pixelfed" => MentionInputProfile {
            label: "Account Name",
            placeholder: "e.g. ranrar",
            helper: "Use account name only (default instance is pixelfed.social).",
            kind: InputKind::Username,
        },
        "discord" => MentionInputProfile {
            label: "User ID",
            placeholder: "e.g. 80351110224678912",
            helper: "Use the numeric Discord user ID.",
            kind: InputKind::NumericId,
        },
        _ => MentionInputProfile {
            label: "Username",
            placeholder: "e.g. ranrar",
            helper: "Use the public username/handle for this platform.",
            kind: InputKind::Username,
        },
    }
}

fn validate_identifier(platform_key: &str, identifier: &str) -> bool {
    let trimmed = identifier.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_USERNAME_LEN {
        return false;
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return false;
    }

    match platform_key {
        "github" => {
            trimmed.len() <= 39
                && !trimmed.starts_with('-')
                && !trimmed.ends_with('-')
                && !trimmed.contains("--")
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-')
        }
        "telegram" => {
            (5..=32).contains(&trimmed.len())
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        "reddit" => {
            (3..=20).contains(&trimmed.len())
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        "linkedin" | "xing" | "quora" => {
            (3..=100).contains(&trimmed.len())
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        }
        "discord" => {
            (17..=20).contains(&trimmed.len()) && trimmed.chars().all(|c| c.is_ascii_digit())
        }
        "bilibili" => {
            trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.parse::<u64>().unwrap_or(0) > 0
        }
        "bluesky" => is_domain_like(trimmed),
        _ => match profile_for_platform(platform_key).kind {
            InputKind::Username => is_handle_like(trimmed),
            InputKind::NumericId => trimmed.chars().all(|c| c.is_ascii_digit()),
            InputKind::DomainHandle => is_domain_like(trimmed),
        },
    }
}

fn fallback_texture() -> gdk::MemoryTexture {
    let bytes = glib::Bytes::from_owned(vec![0u8, 0u8, 0u8, 0u8]);
    gdk::MemoryTexture::new(1, 1, gdk::MemoryFormat::B8g8r8a8Premultiplied, &bytes, 4)
}

fn normalize_svg_stroke_width(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len() + 16);
    let mut cursor = 0usize;

    while let Some(found) = svg[cursor..].find("stroke-width=") {
        let start = cursor + found;
        out.push_str(&svg[cursor..start]);

        let after_key = start + "stroke-width=".len();
        let Some(quote) = svg[after_key..].chars().next() else {
            out.push_str("stroke-width=\"1\"");
            cursor = after_key;
            continue;
        };

        if quote != '"' && quote != '\'' {
            out.push_str("stroke-width=\"1\"");
            cursor = after_key;
            continue;
        }

        let value_start = after_key + quote.len_utf8();
        if let Some(end_rel) = svg[value_start..].find(quote) {
            let value_end = value_start + end_rel;
            out.push_str("stroke-width=");
            out.push(quote);
            out.push('1');
            out.push(quote);
            cursor = value_end + quote.len_utf8();
        } else {
            out.push_str("stroke-width=\"1\"");
            cursor = value_start;
        }
    }

    out.push_str(&svg[cursor..]);
    out
}

fn render_svg_texture(svg: &str, color: &str, icon_size: f64) -> gdk::MemoryTexture {
    let svg_colored = svg.replace("currentColor", color);
    let svg_colored = normalize_svg_stroke_width(&svg_colored);
    let bytes = glib::Bytes::from_owned(svg_colored.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);

    let handle =
        match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(_) => return fallback_texture(),
        };

    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);

    let render_scale = display_scale * 2.0;
    let render_size = (icon_size * render_scale) as i32;

    let mut surface =
        match cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size) {
            Ok(s) => s,
            Err(_) => return fallback_texture(),
        };

    {
        let cr = match cairo::Context::new(&surface) {
            Ok(c) => c,
            Err(_) => return fallback_texture(),
        };

        cr.scale(render_scale, render_scale);
        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
        if renderer.render_document(&cr, &viewport).is_err() {
            return fallback_texture();
        }
    }

    let data = match surface.data() {
        Ok(d) => d.to_vec(),
        Err(_) => return fallback_texture(),
    };

    let bytes = glib::Bytes::from_owned(data);
    gdk::MemoryTexture::new(
        render_size,
        render_size,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    )
}

fn create_platform_button(
    label: &str,
    svg: Option<&'static str>,
    icon_color: &str,
) -> gtk4::Button {
    let button = Button::new();
    button.set_has_frame(false);
    button.add_css_class("marco-mention-platform-btn");

    let row = Box::new(Orientation::Horizontal, 6);
    row.set_halign(Align::Fill);

    if let Some(svg_markup) = svg {
        let picture = Picture::new();
        let texture = render_svg_texture(svg_markup, icon_color, 14.0);
        picture.set_paintable(Some(&texture));
        picture.set_size_request(14, 14);
        picture.add_css_class("marco-mention-platform-icon");
        row.append(&picture);
    }

    let text = Label::new(Some(label));
    text.set_xalign(0.0);
    text.add_css_class("marco-mention-platform-label");
    row.append(&text);

    button.set_child(Some(&row));
    button
}

pub fn show_insert_mention_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    let theme_class = if parent.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let icon_color = if theme_class == "marco-theme-dark" {
        "#c9d1d9"
    } else {
        "#334155"
    };

    let dialog = Window::builder()
        .modal(false)
        .transient_for(parent)
        .default_width(620)
        .default_height(420)
        .resizable(true)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(theme_class);

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert Mention",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert Mention dialog requires a close button");
    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    let root = Box::new(Orientation::Vertical, 0);
    let vbox = Box::new(Orientation::Vertical, 6);
    vbox.add_css_class("marco-dialog-content");
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(0);

    let mention_label = Label::new(Some("Mention"));
    mention_label.set_halign(Align::Start);
    mention_label.add_css_class("marco-dialog-section-label");
    mention_label.add_css_class("marco-dialog-section-label-strong");
    vbox.append(&mention_label);

    let platforms = marco_core::render::plarform_mentions::supported_platforms();
    let columns = 4usize;
    let row_count = platforms.len().div_ceil(columns);
    let platform_area_height = ((row_count * 32) + ((row_count.saturating_sub(1)) * 4) + 10) as i32;

    // Keep enough room for all platform buttons plus input/validation/actions.
    // Scrolling remains available as a fallback on very small displays.
    dialog.set_default_height(platform_area_height + 260);

    let selected_platform = Rc::new(RefCell::new(
        platforms
            .first()
            .map(|p| p.key.to_string())
            .unwrap_or_else(|| "github".to_string()),
    ));

    let platform_buttons: Rc<RefCell<Vec<(String, Button)>>> = Rc::new(RefCell::new(Vec::new()));

    let platform_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(platform_area_height)
        .hexpand(true)
        .vexpand(true)
        .build();

    let grid = gtk4::Grid::new();
    grid.add_css_class("marco-mention-grid");
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);

    for (idx, platform) in platforms.iter().enumerate() {
        let button = create_platform_button(platform.label, platform.svg, icon_color);
        if idx == 0 {
            button.add_css_class("selected");
        }

        let row = (idx / columns) as i32;
        let col = (idx % columns) as i32;
        grid.attach(&button, col, row, 1, 1);

        platform_buttons
            .borrow_mut()
            .push((platform.key.to_string(), button.clone()));
    }

    platform_scroll.set_child(Some(&grid));
    vbox.append(&platform_scroll);

    let field_labels_row = Box::new(Orientation::Horizontal, 8);
    field_labels_row.set_margin_top(2);

    let identifier_label = Label::new(Some("User Name"));
    identifier_label.set_halign(Align::Start);
    identifier_label.set_xalign(0.0);
    identifier_label.set_hexpand(true);
    identifier_label.add_css_class("marco-mention-field-label");

    let real_name_label = Label::new(Some("Real Name"));
    real_name_label.set_halign(Align::Start);
    real_name_label.set_xalign(0.0);
    real_name_label.set_hexpand(true);
    real_name_label.add_css_class("marco-mention-field-label");

    field_labels_row.append(&identifier_label);
    field_labels_row.append(&real_name_label);
    vbox.append(&field_labels_row);

    let fields_row = Box::new(Orientation::Horizontal, 8);
    fields_row.set_margin_top(0);

    let identifier_entry = Entry::new();
    identifier_entry.add_css_class("marco-textfield-entry");
    identifier_entry.set_hexpand(true);

    let display_entry = Entry::new();
    display_entry.add_css_class("marco-textfield-entry");
    display_entry.set_hexpand(true);
    display_entry.set_placeholder_text(Some("Real name (optional)"));

    fields_row.append(&identifier_entry);
    fields_row.append(&display_entry);
    vbox.append(&fields_row);

    let helper_label = Label::new(None);
    helper_label.set_halign(Align::Start);
    helper_label.set_xalign(0.0);
    helper_label.set_wrap(true);
    helper_label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    helper_label.add_css_class("marco-mention-helper");
    vbox.append(&helper_label);

    let validation_note_label = Label::new(None);
    validation_note_label.set_halign(Align::Start);
    validation_note_label.set_xalign(0.0);
    validation_note_label.set_wrap(true);
    validation_note_label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    validation_note_label.add_css_class("marco-mention-validation-note");
    vbox.append(&validation_note_label);

    let valid_button = Button::with_label("Valid");
    valid_button.add_css_class("marco-btn");
    set_valid_button_state(
        &valid_button,
        ValidButtonState::Neutral,
        "Profile check: waiting for input.",
    );

    let cancel_button = Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let insert_button = Button::with_label("Insert");
    insert_button.add_css_class("marco-btn");
    insert_button.add_css_class("suggested-action");

    let button_box = Box::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);
    button_box.append(&cancel_button);
    button_box.append(&insert_button);

    let bottom_frame = gtk4::Frame::new(None);
    bottom_frame.add_css_class("marco-dialog-bottom-frame");
    bottom_frame.set_height_request(48);
    bottom_frame.set_vexpand(false);
    bottom_frame.set_margin_top(2);

    let bottom_inner = Box::new(Orientation::Horizontal, 0);
    bottom_inner.set_margin_start(8);
    bottom_inner.set_margin_end(8);
    bottom_inner.set_margin_top(4);
    bottom_inner.set_margin_bottom(4);
    bottom_inner.set_halign(Align::Fill);
    bottom_inner.set_valign(Align::Center);

    bottom_inner.append(&valid_button);

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bottom_inner.append(&spacer);
    bottom_inner.append(&button_box);
    bottom_frame.set_child(Some(&bottom_inner));

    root.append(&vbox);
    root.append(&bottom_frame);
    dialog.set_child(Some(&root));

    let (profile_check_tx, profile_check_rx) = std::sync::mpsc::channel::<ProfileCheckResult>();
    let pending_check_token = Rc::new(RefCell::new(0_u64));
    let input_change_seq = Rc::new(RefCell::new(0_u64));
    let profile_exists_state = Rc::new(RefCell::new(None::<bool>));

    {
        let pending_check_token = pending_check_token.clone();
        let profile_exists_state = profile_exists_state.clone();
        let valid_button = valid_button.clone();
        glib::timeout_add_local(Duration::from_millis(120), move || {
            while let Ok(result) = profile_check_rx.try_recv() {
                if result.token != *pending_check_token.borrow() {
                    continue;
                }

                *profile_exists_state.borrow_mut() = result.exists;
                let text = match result.exists {
                    Some(true) => format!("Profile check: {}", result.message),
                    Some(false) => format!("Profile check: {}", result.message),
                    None => format!("Profile check: {}", result.message),
                };

                let state = if result.exists == Some(true) {
                    ValidButtonState::Valid
                } else {
                    ValidButtonState::Error
                };
                set_valid_button_state(&valid_button, state, &text);
            }

            glib::ControlFlow::Continue
        });
    }

    let refresh_input_profile = {
        let selected_platform = selected_platform.clone();
        let identifier_entry = identifier_entry.clone();
        let helper_label = helper_label.clone();
        let validation_note_label = validation_note_label.clone();
        let identifier_label = identifier_label.clone();
        let profile_exists_state = profile_exists_state.clone();
        let valid_button = valid_button.clone();

        move || {
            let platform_key = selected_platform.borrow().clone();
            let profile = profile_for_platform(&platform_key);
            identifier_label.set_text(profile.label);
            identifier_entry.set_placeholder_text(Some(profile.placeholder));
            helper_label.set_text(profile.helper);

            let note = validation_helper_note(&platform_key);
            validation_note_label.set_text(note);
            if validation_capability(&platform_key) == ValidationCapability::NotImplemented {
                validation_note_label.add_css_class("marco-mention-validation-note-error");
            } else {
                validation_note_label.remove_css_class("marco-mention-validation-note-error");
            }

            set_valid_button_state(
                &valid_button,
                ValidButtonState::Neutral,
                "Profile check: waiting for input.",
            );
            identifier_entry.set_tooltip_text(Some(profile.label));
            *profile_exists_state.borrow_mut() = None;
        }
    };

    let refresh_validation = {
        let selected_platform = selected_platform.clone();
        let identifier_entry = identifier_entry.clone();
        let insert_button = insert_button.clone();
        let profile_exists_state = profile_exists_state.clone();
        let valid_button = valid_button.clone();

        move || {
            let platform_key = selected_platform.borrow().clone();
            let valid = validate_identifier(&platform_key, &identifier_entry.text());

            insert_button.set_sensitive(valid);

            if !valid {
                *profile_exists_state.borrow_mut() = None;
                set_valid_button_state(
                    &valid_button,
                    ValidButtonState::Neutral,
                    "Profile check: waiting for input.",
                );
            }
        }
    };

    let trigger_live_profile_check = {
        let selected_platform = selected_platform.clone();
        let identifier_entry = identifier_entry.clone();
        let pending_check_token = pending_check_token.clone();
        let profile_check_tx = profile_check_tx.clone();
        let profile_exists_state = profile_exists_state.clone();
        let valid_button = valid_button.clone();

        move || {
            let platform_key = selected_platform.borrow().clone();
            let identifier = identifier_entry.text().trim().to_string();

            if identifier.is_empty() {
                *profile_exists_state.borrow_mut() = None;
                set_valid_button_state(
                    &valid_button,
                    ValidButtonState::Neutral,
                    "Profile check: waiting for input.",
                );
                return;
            }

            if !validate_identifier(&platform_key, &identifier) {
                *profile_exists_state.borrow_mut() = None;
                set_valid_button_state(
                    &valid_button,
                    ValidButtonState::Error,
                    "Profile check: enter a valid value first.",
                );
                return;
            }

            match validation_capability(&platform_key) {
                ValidationCapability::Unsupported => {
                    *profile_exists_state.borrow_mut() = None;
                    set_valid_button_state(
                        &valid_button,
                        ValidButtonState::Error,
                        "Profile check: this platform does not support reliable validation.",
                    );
                    return;
                }
                ValidationCapability::NotImplemented => {
                    *profile_exists_state.borrow_mut() = None;
                    set_valid_button_state(
                        &valid_button,
                        ValidButtonState::Error,
                        "Profile check: not implemented for this platform yet.",
                    );
                    return;
                }
                ValidationCapability::Supported => {}
            }

            *pending_check_token.borrow_mut() += 1;
            let token = *pending_check_token.borrow();
            *profile_exists_state.borrow_mut() = None;
            set_valid_button_state(
                &valid_button,
                ValidButtonState::Neutral,
                "Profile check: checking…",
            );

            let profile_check_tx = profile_check_tx.clone();
            std::thread::spawn(move || {
                let mut result = check_profile_exists(&platform_key, &identifier);
                result.token = token;
                let _ = profile_check_tx.send(result);
            });
        }
    };

    for (platform_key, button) in platform_buttons.borrow().iter() {
        let platform_key = platform_key.clone();
        let selected_platform = selected_platform.clone();
        let platform_buttons = platform_buttons.clone();
        let refresh_input_profile = refresh_input_profile.clone();
        let refresh_validation = refresh_validation.clone();
        let trigger_live_profile_check = trigger_live_profile_check.clone();

        button.connect_clicked(move |_| {
            *selected_platform.borrow_mut() = platform_key.clone();

            for (key, candidate) in platform_buttons.borrow().iter() {
                if *key == platform_key {
                    candidate.add_css_class("selected");
                } else {
                    candidate.remove_css_class("selected");
                }
            }

            refresh_input_profile();
            refresh_validation();
            trigger_live_profile_check();
        });
    }

    {
        let refresh_validation = refresh_validation.clone();
        let trigger_live_profile_check = trigger_live_profile_check.clone();
        let input_change_seq = input_change_seq.clone();
        identifier_entry.connect_changed(move |_| {
            refresh_validation();

            *input_change_seq.borrow_mut() += 1;
            let current_seq = *input_change_seq.borrow();

            let trigger_live_profile_check = trigger_live_profile_check.clone();
            let input_change_seq = input_change_seq.clone();
            glib::timeout_add_local(Duration::from_millis(450), move || {
                // If new keystrokes happened after this timer was scheduled,
                // do nothing and let only the newest timer trigger validation.
                if current_seq == *input_change_seq.borrow() {
                    trigger_live_profile_check();
                }
                glib::ControlFlow::Break
            });
        });
    }

    {
        let selected_platform = selected_platform.clone();
        let identifier_entry = identifier_entry.clone();
        let profile_exists_state = profile_exists_state.clone();
        valid_button.connect_clicked(move |_| {
            if *profile_exists_state.borrow() != Some(true) {
                return;
            }

            let platform_key = selected_platform.borrow().clone();
            let identifier = identifier_entry.text().trim().to_string();

            if !validate_identifier(&platform_key, &identifier) {
                return;
            }

            let Some(url) =
                marco_core::render::plarform_mentions::profile_url(&platform_key, &identifier)
            else {
                return;
            };

            let _ = gio::AppInfo::launch_default_for_uri(&url, None::<&gio::AppLaunchContext>);
        });
    }

    refresh_input_profile();
    refresh_validation();
    trigger_live_profile_check();

    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    {
        let selected_platform = selected_platform.clone();
        let identifier_entry = identifier_entry.clone();
        let display_entry = display_entry.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let dialog_weak = dialog_weak.clone();

        insert_button.connect_clicked(move |_| {
            let platform_key = selected_platform.borrow().clone();
            let identifier = identifier_entry.text().trim().to_string();
            if !validate_identifier(&platform_key, &identifier) {
                return;
            }

            let display = display_entry.text().trim().to_string();
            let mention_markdown = if display.is_empty() {
                format!("@{}[{}]", identifier, platform_key)
            } else {
                format!("@{}[{}]({})", identifier, platform_key, display)
            };

            insert_mention_at_cursor(&editor_buffer, &editor_view, &mention_markdown);

            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        cancel_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        close_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let key_controller = gtk4::EventControllerKey::new();
        let dialog_weak = dialog_weak.clone();
        key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
            if key == gtk4::gdk::Key::Escape {
                if let Some(dialog) = dialog_weak.upgrade() {
                    dialog.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        dialog.add_controller(key_controller);
    }

    identifier_entry.grab_focus();
    dialog.present();
}

fn insert_mention_at_cursor(buffer: &Buffer, view: &View, mention_text: &str) {
    let insert_mark = buffer.get_insert();
    let mut cursor_iter = buffer.iter_at_mark(&insert_mark);

    let line_start = {
        let mut iter = cursor_iter;
        iter.set_line_offset(0);
        iter
    };

    let line_end = {
        let mut iter = cursor_iter;
        if !iter.ends_line() {
            iter.forward_to_line_end();
        }
        iter
    };

    let current_line = buffer.text(&line_start, &line_end, false);
    let indent = current_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    let indented_text = if indent.is_empty() {
        mention_text.to_string()
    } else {
        format!("{}{}", indent, mention_text)
    };

    buffer.insert(&mut cursor_iter, &indented_text);

    let insert_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&insert_mark);
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_validate_identifier_basic_username() {
        assert!(validate_identifier("github", "ranrar"));
        assert!(validate_identifier("github", "ran-rar"));
        assert!(!validate_identifier("github", "ranrar.dev"));
        assert!(!validate_identifier("github", "ranrar/dev"));
    }

    #[test]
    fn smoke_test_validate_identifier_numeric_id_platforms() {
        assert!(validate_identifier("discord", "80351110224678912"));
        assert!(!validate_identifier("discord", "ranrar"));
        assert!(validate_identifier("bilibili", "2"));
    }

    #[test]
    fn smoke_test_validate_identifier_domain_handle() {
        assert!(validate_identifier("bluesky", "bsky.app"));
        assert!(!validate_identifier("bluesky", "ranrar"));
    }
}
