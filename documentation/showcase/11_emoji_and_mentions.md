# Emoji Shortcodes and Platform Mentions

---

## Emoji Shortcodes

Write `:shortcode:` to insert an emoji. Only recognized shortcodes are converted — unknown ones stay as literal text.

### Common Shortcodes

:smile: :joy: :heart: :thumbsup: :thumbsdown: :tada: :rocket: :fire:

:warning: :x: :white_check_mark: :information_source: :bug: :memo:

:coffee: :eyes: :star: :sparkles: :100: :muscle: :wave:

### Development and Technical

:wrench: :gear: :hammer: :computer: :keyboard: :floppy_disk: :inbox_tray:

:lock: :key: :shield: :zap: :chart_with_upwards_trend: :microscope:

### Communication and Status

:speech_balloon: :mailbox: :calendar: :clock1: :hourglass: :bell: :loudspeaker:

:green_circle: :yellow_circle: :red_circle: :white_circle:

### Shortcodes in Context

Deployment status: :rocket: **v0.9.0 released** :tada:

Bug tracker: :bug: Issue #42 is :white_check_mark: fixed.

> [!TIP]
> :bulb: Emoji shortcodes work inside blockquotes, tables, and most inline contexts.

Code spans keep shortcodes literal: `:rocket:` ← stays as text.

```
:rocket: inside a fenced code block also stays literal
```

### In Tables

| Status | Symbol | Meaning |
|--------|--------|---------|
| Done | :white_check_mark: | Completed successfully |
| In progress | :hourglass: | Currently being worked on |
| Blocked | :red_circle: | Waiting on a dependency |
| Cancelled | :x: | No longer planned |

### Unknown Shortcodes Stay Literal

:this_is_not_a_real_emoji: — stays as-is.

:another_unknown: — also stays literal.

---

## Platform Mentions

Mention users on supported platforms with `@username[platform]` or `@username[platform](Display Name)`.

### Basic Mentions

@ranrar[github]

@torvalds[github]

@dribbble[dribbble]

### Mentions with Display Names

The optional display name replaces the username in the rendered link:

@ranrar[github](Kim)

@john[twitter](John Doe)

@jane[linkedin](Jane Smith)

### Supported Platforms

:::tab
@tab Social
| Platform | Syntax | Example |
|----------|--------|---------|
| GitHub | `@user[github]` | @ranrar[github] |
| GitLab | `@user[gitlab]` | @user[gitlab] |
| Codeberg | `@user[codeberg]` | @user[codeberg] |
| Twitter/X | `@user[twitter]` | @user[twitter] |
| Mastodon | `@user[mastodon]` | @user[mastodon] |
| Bluesky | `@user[bluesky]` | @user[bluesky] |

@tab Professional
| Platform | Syntax | Example |
|----------|--------|---------|
| LinkedIn | `@user[linkedin]` | @user[linkedin] |
| Xing | `@user[xing]` | @user[xing] |
| Medium | `@user[medium]` | @user[medium] |
| Dribbble | `@user[dribbble]` | @user[dribbble] |
| Behance | `@user[behance]` | @user[behance] |

@tab Communities
| Platform | Syntax | Example |
|----------|--------|---------|
| Reddit | `@user[reddit]` | @user[reddit] |
| Discord | `@user[discord]` | @user[discord] |
| Telegram | `@user[telegram]` | @user[telegram] |
| YouTube | `@user[youtube]` | @user[youtube] |
| Twitch | `@user[twitch]` | @user[twitch] |
:::

### Unknown Platforms Render Without a Link

Unsupported platforms render as styled text, not a hyperlink:

@someone[wechat](WeChat User)

@user[unknownplatform]

### Mentions Mid-Sentence

Hello @ranrar[github](Kim), welcome to the project!

Thanks to @alice[github] and @bob[gitlab](Bobby) for the contributions.

### Mentions Near Punctuation

(@ranrar[github])

See @ranrar[github].

Message @ranrar[github], if you have questions.

### Code Spans Prevent Mention Parsing

`@ranrar[github]` ← stays literal.

```
@ranrar[github] inside a code block stays literal too
```

### Not a Mention (No Platform)

`a@b.com` — an email address, not a mention — stays literal.

`@justname` — no `[platform]` bracket, stays literal.
