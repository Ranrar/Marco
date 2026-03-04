//! Platform mention mapping and URL generation.
//!
//! This module contains the supported platform list and logic for turning a
//! `(platform, username)` pair into a profile URL.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MentionPlatform {
    pub key: &'static str,
    pub label: &'static str,
    pub svg: Option<&'static str>,
}

const SVG_GITHUB: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-github'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M9 19c-4.3 1.4 -4.3 -2.5 -6 -3m12 5v-3.5c0 -1 .1 -1.4 -.5 -2c2.8 -.3 5.5 -1.4 5.5 -6a4.6 4.6 0 0 0 -1.3 -3.2a4.2 4.2 0 0 0 -.1 -3.2s-1.1 -.3 -3.5 1.3a12.3 12.3 0 0 0 -6.2 0c-2.4 -1.6 -3.5 -1.3 -3.5 -1.3a4.2 4.2 0 0 0 -.1 3.2a4.6 4.6 0 0 0 -1.3 3.2c0 4.6 2.7 5.7 5.5 6c-.6 .6 -.6 1.2 -.5 2v3.5' /></svg>"#;
const SVG_GITLAB: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-gitlab'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M21 14l-9 7l-9 -7l3 -11l3 7h6l3 -7l3 11' /></svg>"#;
const SVG_BITBUCKET: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-bitbucket'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3.648 4a.64 .64 0 0 0 -.64 .744l3.14 14.528c.07 .417 .43 .724 .852 .728h10a.644 .644 0 0 0 .642 -.539l3.35 -14.71a.641 .641 0 0 0 -.64 -.744l-16.704 -.007' /><path d='M14 15h-4l-1 -6h6l-1 6' /></svg>"#;
const SVG_X: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-x'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M4 4l11.733 16h4.267l-11.733 -16z' /><path d='M4 20l6.768 -6.768m2.46 -2.46l6.772 -6.772' /></svg>"#;
const SVG_TWITTER: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-twitter'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M22 4.01c-1 .49 -1.98 .689 -3 .99c-1.121 -1.265 -2.783 -1.335 -4.38 -.737s-2.643 2.06 -2.62 3.737v1c-3.245 .083 -6.135 -1.395 -8 -4c0 0 -4.182 7.433 4 11c-1.872 1.247 -3.739 2.088 -6 2c3.308 1.803 6.913 2.423 10.034 1.517c3.58 -1.04 6.522 -3.723 7.651 -7.742a13.84 13.84 0 0 0 .497 -3.753c0 -.249 1.51 -2.772 1.818 -4.013l0 .001' /></svg>"#;
const SVG_REDDIT: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-reddit'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M12 8c2.648 0 5.028 .826 6.675 2.14a2.5 2.5 0 0 1 2.326 4.36c0 3.59 -4.03 6.5 -9 6.5c-4.875 0 -8.845 -2.8 -9 -6.294l-1 -.206a2.5 2.5 0 0 1 2.326 -4.36c1.646 -1.313 4.026 -2.14 6.674 -2.14l.999 0' /><path d='M12 8l1 -5l6 1' /><path d='M18 4a1 1 0 1 0 2 0a1 1 0 1 0 -2 0' /><path d='M8.5 13a.5 .5 0 1 0 1 0a.5 .5 0 1 0 -1 0' fill='currentColor' /><path d='M14.5 13a.5 .5 0 1 0 1 0a.5 .5 0 1 0 -1 0' fill='currentColor' /><path d='M10 17c.667 .333 1.333 .5 2 .5s1.333 -.167 2 -.5' /></svg>"#;
const SVG_INSTAGRAM: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-instagram'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M4 8a4 4 0 0 1 4 -4h8a4 4 0 0 1 4 4v8a4 4 0 0 1 -4 4h-8a4 4 0 0 1 -4 -4z' /><path d='M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0' /><path d='M16.5 7.5v.01' /></svg>"#;
const SVG_YOUTUBE: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-youtube'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M2 8a4 4 0 0 1 4 -4h12a4 4 0 0 1 4 4v8a4 4 0 0 1 -4 4h-12a4 4 0 0 1 -4 -4z' /><path d='M10 9l5 3l-5 3z' /></svg>"#;
const SVG_LINKEDIN: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-linkedin'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M8 11v5' /><path d='M8 8v.01' /><path d='M12 16v-5' /><path d='M16 16v-3a2 2 0 1 0 -4 0' /><path d='M3 7a4 4 0 0 1 4 -4h10a4 4 0 0 1 4 4v10a4 4 0 0 1 -4 4h-10a4 4 0 0 1 -4 -4z' /></svg>"#;
const SVG_FACEBOOK: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-facebook'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M7 10v4h3v7h4v-7h3l1 -4h-4v-2a1 1 0 0 1 1 -1h3v-4h-3a5 5 0 0 0 -5 5v2h-3' /></svg>"#;
const SVG_DISCORD: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-discord'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M8 12a1 1 0 1 0 2 0a1 1 0 0 0 -2 0' /><path d='M14 12a1 1 0 1 0 2 0a1 1 0 0 0 -2 0' /><path d='M15.5 17c0 1 1.5 3 2 3c1.5 0 2.833 -1.667 3.5 -3c.667 -1.667 .5 -5.833 -1.5 -11.5c-1.457 -1.015 -3 -1.34 -4.5 -1.5l-.972 1.923a11.913 11.913 0 0 0 -4.053 0l-.975 -1.923c-1.5 .16 -3.043 .485 -4.5 1.5c-2 5.667 -2.167 9.833 -1.5 11.5c.667 1.333 2 3 3.5 3c.5 0 2 -2 2 -3' /><path d='M7 16.5c3.5 1 6.5 1 10 0' /></svg>"#;
const SVG_TELEGRAM: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-telegram'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M15 10l-4 4l6 6l4 -16l-18 7l4 2l2 6l3 -4' /></svg>"#;
const SVG_TWITCH: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-twitch'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M4 5v11a1 1 0 0 0 1 1h2v4l4 -4h5.584c.266 0 .52 -.105 .707 -.293l2.415 -2.414c.187 -.188 .293 -.442 .293 -.708v-8.585a1 1 0 0 0 -1 -1h-14a1 1 0 0 0 -1 1l.001 0' /><path d='M16 8v4' /><path d='M12 8v4' /></svg>"#;
const SVG_MEDIUM: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-medium'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M4 6a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2z' /><path d='M8 9h1l3 3l3 -3h1' /><path d='M8 15h2' /><path d='M14 15h2' /><path d='M9 9v6' /><path d='M15 9v6' /></svg>"#;
const SVG_PINTEREST: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-pinterest'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M8 20l4 -9' /><path d='M10.7 14c.437 1.263 1.43 2 2.55 2c2.071 0 3.75 -1.554 3.75 -4a5 5 0 1 0 -9.7 1.7' /><path d='M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0' /></svg>"#;
const SVG_DRIBBBLE: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-dribbble'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0' /><path d='M9 3.6c5 6 7 10.5 7.5 16.2' /><path d='M6.4 19c3.5 -3.5 6 -6.5 14.5 -6.4' /><path d='M3.1 10.75c5 0 9.814 -.38 15.314 -5' /></svg>"#;
const SVG_SNAPCHAT: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-snapchat'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M16.882 7.842a4.882 4.882 0 0 0 -9.764 0c0 4.273 -.213 6.409 -4.118 8.118c2 .882 2 .882 3 3c3 0 4 2 6 2s3 -2 6 -2c1 -2.118 1 -2.118 3 -3c-3.906 -1.709 -4.118 -3.845 -4.118 -8.118m-13.882 8.119c4 -2.118 4 -4.118 1 -7.118m17 7.118c-4 -2.118 -4 -4.118 -1 -7.118' /></svg>"#;
const SVG_TIKTOK: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-tiktok'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M21 7.917v4.034a9.948 9.948 0 0 1 -5 -1.951v4.5a6.5 6.5 0 1 1 -8 -6.326v4.326a2.5 2.5 0 1 0 4 2v-11.5h4.083a6.005 6.005 0 0 0 4.917 4.917' /></svg>"#;
const SVG_THREADS: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-threads'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M19 7.5c-1.333 -3 -3.667 -4.5 -7 -4.5c-5 0 -8 2.5 -8 9s3.5 9 8 9s7 -3 7 -5s-1 -5 -7 -5c-2.5 0 -3 1.25 -3 2.5c0 1.5 1 2.5 2.5 2.5c2.5 0 3.5 -1.5 3.5 -5s-2 -4 -3 -4s-1.833 .333 -2.5 1' /></svg>"#;
const SVG_SOUNDCLOUD: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-soundcloud'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M17 11h1c1.38 0 3 1.274 3 3c0 1.657 -1.5 3 -3 3l-6 0v-10c3 0 4.5 1.5 5 4' /><path d='M9 8l0 9' /><path d='M6 17l0 -7' /><path d='M3 16l0 -2' /></svg>"#;
const SVG_VK: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-vk'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M14 19h-4a8 8 0 0 1 -8 -8v-5h4v5a4 4 0 0 0 4 4v-9h4v4.5l.03 0a4.531 4.531 0 0 0 3.97 -4.496h4l-.342 1.711a6.858 6.858 0 0 1 -3.658 4.789a5.34 5.34 0 0 1 3.566 4.111l.434 2.389h-4a4.531 4.531 0 0 0 -3.97 -4.496v4.5l-.03 -.008' /></svg>"#;
const SVG_TUMBLR: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-tumblr'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M14 21h4v-4h-4v-6h4v-4h-4v-4h-4v1a3 3 0 0 1 -3 3h-1v4h4v6a4 4 0 0 0 4 4' /></svg>"#;
const SVG_MASTODON: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-mastodon'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M18.648 15.254c-1.816 1.763 -6.648 1.626 -6.648 1.626a18.262 18.262 0 0 1 -3.288 -.256c1.127 1.985 4.12 2.81 8.982 2.475c-1.945 2.013 -13.598 5.257 -13.668 -7.636l-.026 -1.154c0 -3.036 .023 -4.115 1.352 -5.633c1.671 -1.91 6.648 -1.666 6.648 -1.666s4.977 -.243 6.648 1.667c1.329 1.518 1.352 2.597 1.352 5.633s-.456 4.074 -1.352 4.944' /><path d='M12 11.204v-2.926c0 -1.258 -.895 -2.278 -2 -2.278s-2 1.02 -2 2.278v4.722m4 -4.722c0 -1.258 .895 -2.278 2 -2.278s2 1.02 2 2.278v4.722' /></svg>"#;
const SVG_BLUESKY: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-bluesky'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M6.335 5.144c-1.654 -1.199 -4.335 -2.127 -4.335 .826c0 .59 .35 4.953 .556 5.661c.713 2.463 3.13 2.75 5.444 2.369c-4.045 .665 -4.889 3.208 -2.667 5.41c1.03 1.018 1.913 1.59 2.667 1.59c2 0 3.134 -2.769 3.5 -3.5c.333 -.667 .5 -1.167 .5 -1.5c0 .333 .167 .833 .5 1.5c.366 .731 1.5 3.5 3.5 3.5c.754 0 1.637 -.571 2.667 -1.59c2.222 -2.203 1.378 -4.746 -2.667 -5.41c2.314 .38 4.73 .094 5.444 -2.369c.206 -.708 .556 -5.072 .556 -5.661c0 -2.953 -2.68 -2.025 -4.335 -.826c-2.293 1.662 -4.76 5.048 -5.665 6.856c-.905 -1.808 -3.372 -5.194 -5.665 -6.856' /></svg>"#;
const SVG_CODEBERG: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-codeberg'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0'/><path d='M9 7l6 0l-6 10l6 0'/></svg>"#;
const SVG_XING: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-xing'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M16 21l-4 -7l6.5 -11'/><path d='M7 7l2 3.5l-3 4.5'/></svg>"#;
const SVG_MIXCLOUD: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-mixcloud'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M4 12a8 8 0 0 1 16 0'/><path d='M7 12a5 5 0 0 1 10 0'/><path d='M10 12v3'/><path d='M14 12v3'/></svg>"#;
const SVG_QUORA: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-quora'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M12 3c4.97 0 9 3.582 9 8c0 4.419 -4.03 8 -9 8c-1.042 0 -2.05 -.156 -3.009 -.46c-1.098 1.734 -2.806 2.882 -4.731 2.882c-.381 0 -.722 -.04 -1.063 -.117c.766 -1.027 1.176 -2.251 1.176 -3.584c0 -.412 -.042 -.817 -.122 -1.207c-1.502 -1.407 -2.251 -3.335 -2.251 -5.514c0 -4.418 4.03 -8 9 -8z'/></svg>"#;
const SVG_MYSPACE: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-myspace'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><circle cx='8' cy='8' r='2'/><circle cx='16' cy='8' r='2'/><circle cx='12' cy='16' r='2'/><path d='M8 10a8 8 0 0 1 8 0'/></svg>"#;
const SVG_9GAG: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-9gag'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><circle cx='12' cy='12' r='8'/><path d='M9 12a3 3 0 0 0 6 0'/><path d='M9 9a1 1 0 1 0 0 -2a1 1 0 0 0 0 2'/><path d='M15 9a1 1 0 1 0 0 -2a1 1 0 0 0 0 2'/></svg>"#;
const SVG_LIKEE: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-likee'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M12 3c5 0 7 2 7 7v4c0 5 -2 7 -7 7h-2c-5 0 -7 -2 -7 -7v-4c0 -5 2 -7 7 -7h2z'/><path d='M9 10l6 4l-6 4v-8z' fill='currentColor'/></svg>"#;
const SVG_ZHIHU: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-zhihu'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 6h18v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-12z'/><path d='M8 10l3 3l-3 3'/><path d='M16 10l-3 3l3 3'/></svg>"#;
const SVG_BILIBILI: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-bilibili'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M2 8l2 -2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2 -2v-12l2 2'/><path d='M7 12h10'/><path d='M7 16h10'/></svg>"#;
const SVG_TIEBA: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-tieba'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><circle cx='6' cy='9' r='2'/><circle cx='18' cy='9' r='2'/><circle cx='12' cy='5' r='2'/><circle cx='12' cy='18' r='2'/><path d='M7 9l5 9l5 -9'/></svg>"#;
const SVG_PIXELFED: &str = r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-brand-pixelfed'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><circle cx='12' cy='12' r='9'/><path d='M8 14l2 -3l2 3'/><path d='M14 14l2 -3l2 3'/></svg>"#;

const SUPPORTED_MENTION_PLATFORMS: &[MentionPlatform] = &[
    MentionPlatform {
        key: "github",
        label: "GitHub",
        svg: Some(SVG_GITHUB),
    },
    MentionPlatform {
        key: "gitlab",
        label: "GitLab",
        svg: Some(SVG_GITLAB),
    },
    MentionPlatform {
        key: "bitbucket",
        label: "Bitbucket",
        svg: Some(SVG_BITBUCKET),
    },
    MentionPlatform {
        key: "codeberg",
        label: "Codeberg",
        svg: Some(SVG_CODEBERG),
    },
    MentionPlatform {
        key: "x",
        label: "X",
        svg: Some(SVG_X),
    },
    MentionPlatform {
        key: "twitter",
        label: "Twitter",
        svg: Some(SVG_TWITTER),
    },
    MentionPlatform {
        key: "reddit",
        label: "Reddit",
        svg: Some(SVG_REDDIT),
    },
    MentionPlatform {
        key: "instagram",
        label: "Instagram",
        svg: Some(SVG_INSTAGRAM),
    },
    MentionPlatform {
        key: "snapchat",
        label: "Snapchat",
        svg: Some(SVG_SNAPCHAT),
    },
    MentionPlatform {
        key: "tiktok",
        label: "TikTok",
        svg: Some(SVG_TIKTOK),
    },
    MentionPlatform {
        key: "youtube",
        label: "YouTube",
        svg: Some(SVG_YOUTUBE),
    },
    MentionPlatform {
        key: "linkedin",
        label: "LinkedIn",
        svg: Some(SVG_LINKEDIN),
    },
    MentionPlatform {
        key: "xing",
        label: "XING",
        svg: Some(SVG_XING),
    },
    MentionPlatform {
        key: "facebook",
        label: "Facebook",
        svg: Some(SVG_FACEBOOK),
    },
    MentionPlatform {
        key: "threads",
        label: "Threads",
        svg: Some(SVG_THREADS),
    },
    MentionPlatform {
        key: "twitch",
        label: "Twitch",
        svg: Some(SVG_TWITCH),
    },
    MentionPlatform {
        key: "soundcloud",
        label: "SoundCloud",
        svg: Some(SVG_SOUNDCLOUD),
    },
    MentionPlatform {
        key: "mixcloud",
        label: "Mixcloud",
        svg: Some(SVG_MIXCLOUD),
    },
    MentionPlatform {
        key: "telegram",
        label: "Telegram",
        svg: Some(SVG_TELEGRAM),
    },
    MentionPlatform {
        key: "vk",
        label: "VK",
        svg: Some(SVG_VK),
    },
    MentionPlatform {
        key: "pinterest",
        label: "Pinterest",
        svg: Some(SVG_PINTEREST),
    },
    MentionPlatform {
        key: "medium",
        label: "Medium",
        svg: Some(SVG_MEDIUM),
    },
    MentionPlatform {
        key: "tumblr",
        label: "Tumblr",
        svg: Some(SVG_TUMBLR),
    },
    MentionPlatform {
        key: "quora",
        label: "Quora",
        svg: Some(SVG_QUORA),
    },
    MentionPlatform {
        key: "myspace",
        label: "Myspace",
        svg: Some(SVG_MYSPACE),
    },
    MentionPlatform {
        key: "dribbble",
        label: "Dribbble",
        svg: Some(SVG_DRIBBBLE),
    },
    MentionPlatform {
        key: "9gag",
        label: "9GAG",
        svg: Some(SVG_9GAG),
    },
    MentionPlatform {
        key: "bluesky",
        label: "Bluesky",
        svg: Some(SVG_BLUESKY),
    },
    MentionPlatform {
        key: "likee",
        label: "Likee",
        svg: Some(SVG_LIKEE),
    },
    MentionPlatform {
        key: "zhihu",
        label: "Zhihu",
        svg: Some(SVG_ZHIHU),
    },
    MentionPlatform {
        key: "bilibili",
        label: "Bilibili",
        svg: Some(SVG_BILIBILI),
    },
    MentionPlatform {
        key: "tieba",
        label: "Baidu Tieba",
        svg: Some(SVG_TIEBA),
    },
    MentionPlatform {
        key: "mastodon",
        label: "Mastodon",
        svg: Some(SVG_MASTODON),
    },
    MentionPlatform {
        key: "pixelfed",
        label: "Pixelfed",
        svg: Some(SVG_PIXELFED),
    },
    MentionPlatform {
        key: "discord",
        label: "Discord",
        svg: Some(SVG_DISCORD),
    },
];

/// Returns canonical mention platforms used by renderer and UI integrations.
pub fn supported_platforms() -> &'static [MentionPlatform] {
    SUPPORTED_MENTION_PLATFORMS
}

/// Returns a Tabler SVG logo for known platforms.
///
/// If a platform does not have a mapped Tabler brand icon yet, returns `None`.
pub fn platform_logo_svg(platform: &str) -> Option<&'static str> {
    let p = platform.trim().to_ascii_lowercase();
    supported_platforms()
        .iter()
        .find(|platform| platform.key == p)
        .and_then(|platform| platform.svg)
}

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
    fn smoke_test_supported_platforms_not_empty() {
        assert!(!supported_platforms().is_empty());
    }

    #[test]
    fn smoke_test_platform_logo_svg_known_platform() {
        assert!(platform_logo_svg("github").is_some());
    }

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
