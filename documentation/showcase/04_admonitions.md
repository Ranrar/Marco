# Admonitions and Callouts

Marco supports two admonition syntaxes: GitHub-style alerts and a custom emoji-header form.

---

## GitHub-Style Alerts

Place the marker on the first line of a blockquote. Five standard types are supported.

> [!NOTE]
> Use this for supplementary information that the reader should be aware of, even when skimming.

> [!TIP]
> Use this for helpful hints and shortcuts that improve the experience.

> [!IMPORTANT]
> Use this for information that is critical for the reader to succeed.

> [!WARNING]
> Use this for information about potentially dangerous outcomes or destructive operations.

> [!CAUTION]
> Use this for actions that may have unexpected or undesired consequences.

---

## Multi-line Alert Content

Alert body can span multiple lines and contain full Markdown:

> [!NOTE]
> This note has **bold text**, *italic text*, and `inline code`.
>
> It also has a second paragraph and a list:
>
> - Step one: verify your config
> - Step two: restart the service
> - Step three: check the logs

> [!WARNING]
> Running this command will **permanently delete** all data in the target directory.
>
> Make sure you have a backup before proceeding:
>
> ```bash
> rm -rf /path/to/target
> ```

---

## Custom Emoji Header (Marco Extension)

Use `> [:emoji: Title]` to create an admonition with a custom emoji icon and a free-form title. The body follows on the next lines.

> [:rocket: Getting Started]
> Open a Markdown file or create a new one. The live preview updates as you type.

> [:bulb: Pro Tip]
> Use **Ctrl+Shift+P** to open the command palette and quickly access formatting actions.

> [:warning: Deprecated API]
> This function will be removed in version 2.0. Migrate to `new_function()` before upgrading.

> [:books: Further Reading]
> See the [CommonMark spec](https://spec.commonmark.org) and the [GFM spec](https://github.github.com/gfm/) for the underlying standards.

> [:fire: Breaking Change]
> The settings file format changed in 0.8.0. Copy `settings_org.ron` over your old config to reset defaults.

---

## Practical Examples

> [!TIP]
> Press **Ctrl+D** to duplicate the current line.

> [!IMPORTANT]
> All passwords must be at least 12 characters and include a number and a symbol.

> [!NOTE]
> `cargo test --workspace` runs all tests across the workspace. Use `-- --nocapture` to see `println!` output during tests.

> [!CAUTION]
> Deleting a setting file is irreversible. Export your settings first via **Settings -> Export**.

> [:information_source: Version Note]
> This feature requires Marco 0.9.0 or later. Check **Help -> About** to confirm your version.

---

## Alerts Inside Lists

Alerts at the top level of a document render as styled callouts. Inside a list, they fall back to standard blockquote styling:

1. First, back up your data.

   > [!WARNING]
   > Skip this step only if you're working on a disposable environment.

2. Apply the migration.
3. Verify the output.
