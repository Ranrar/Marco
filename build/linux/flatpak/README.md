# Marco and Polo as Flatpaks

Marco and Polo ship as **two separate Flatpaks**, built by two separate scripts
from two separate manifests. They are independent: either installs and runs
without the other.

|  | Marco | Polo |
|---|---|---|
| Role | editor | viewer |
| App ID | `io.github.ranrar.Marco` | `io.github.ranrar.Marco.Polo` |
| Command | `markdowncomposer` | `markdownviewer` |
| Build script | `build_flatpak_marco.sh` | `build_flatpak_polo.sh` |
| Files | `marco/` | `polo/` |
| D-Bus service file | yes — it is activatable | no — it is the caller |

They were one Flatpak until August 2026. They were separated because the
platform assumes **one application per app ID**, and two desktop files under a
single ID collide irreparably in anything that maps a running window back to an
application name.

---

## 1. Layout

```
build/linux/flatpak/
├── README.md                     # this file
├── _common.sh                    # shared build machinery; sourced, not run
├── build_flatpak_marco.sh        # Marco only
├── build_flatpak_polo.sh         # Polo only
├── cargo-sources.json            # vendored crates — Flathub builds only
├── marco/
│   ├── io.github.ranrar.Marco.yml            # local manifest
│   ├── io.github.ranrar.Marco.flathub.yml    # offline/Flathub manifest
│   ├── io.github.ranrar.Marco.desktop
│   ├── io.github.ranrar.Marco.service        # D-Bus activation
│   ├── io.github.ranrar.Marco.metainfo.xml
│   └── icons/io.github.ranrar.Marco.png      # exactly 256×256
├── polo/
│   ├── io.github.ranrar.Marco.Polo.yml
│   ├── io.github.ranrar.Marco.Polo.flathub.yml
│   ├── io.github.ranrar.Marco.Polo.desktop
│   ├── io.github.ranrar.Marco.Polo.metainfo.xml
│   └── icons/io.github.ranrar.Marco.Polo.png
├── _build/{marco,polo}/          # gitignored — flatpak-builder working dirs
└── _repo/{marco,polo}/           # gitignored — OSTree repos
```

Each app gets its own `_build` and `_repo` subdirectory so the two never trample
each other. Both parents are gitignored, so the build scripts create them; the
manifests' `skip:` lists exclude them from the source copy at directory level,
which covers the per-app subdirectories automatically.

**The manifests hardcode their own directory name** in every `install` command
(`build/linux/flatpak/marco/…`). Renaming this directory means updating those
paths — the build scripts' `--check` catches it and fails early rather than
letting the build die inside the sandbox.

### Why `io.github.ranrar.Marco.Polo` has five components

It looks wrong and is not. Flathub derives an ownership URL from components 2
and 3 only, whatever the ID's length, so this maps to `github.com/ranrar/marco`
— a repository that exists. `io.github.ranrar.Polo` would map to
`github.com/ranrar/polo`, which does not.

It is also the ID Polo already registered under while the two shared one
Flatpak, so its runtime identity did not change across the split.
`polo/src/main.rs::app_id()` must keep returning it under Flatpak: the plain
`APP_ID` is `io.github.ranrar.Polo`, and a GApplication ID that does not match
the sandbox ID makes WebKit's accessibility check reject the WebView at startup.

---

## 2. Prerequisites

```bash
flatpak install -y flathub org.flatpak.Builder
flatpak install -y flathub org.gnome.Platform//50
flatpak install -y flathub org.gnome.Sdk//50
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
```

`--check` on either script verifies all of these and exits.

**No extra Flatpak modules are needed.** `org.gnome.Platform//50` already ships
`libgtk-4.so.1`, `libgtksourceview-5.so.0`, `libwebkitgtk-6.0.so.4`,
`libjavascriptcoregtk-6.0.so.1`, `libsoup-3.0`, `librsvg-2` and `libfontconfig`
— the entire `Depends:` line from `build/linux/debian/build_deb.sh`. Only the
app itself has to be built.

**The Rust pin is not honoured inside the build.** `rust-toolchain.toml` pins
1.94.1; the SDK extension ships a fixed rustc (currently 1.98.0) and cannot run
`rustup` offline. Since the pin is a floor rather than a ceiling, both manifests
delete `rust-toolchain.toml` in a build command and use the SDK's compiler.
Re-check that when the runtime version moves.

---

## 3. Build and install

From anywhere in the workspace:

```bash
bash build/linux/flatpak/build_flatpak_marco.sh
bash build/linux/flatpak/build_flatpak_polo.sh
```

Each script builds, lints, installs into the **user** installation, and writes a
single-file bundle to `build/installer/`:

```
build/installer/marco_0.25.0_linux_amd64.flatpak
build/installer/polo_0.25.0_linux_amd64.flatpak
```

Options, identical on both:

| Flag | Effect |
|---|---|
| `--check` | Verify build dependencies and required files, then exit |
| `--no-install` | Build, but do not install |
| `--no-bundle` | Build (and install), but emit no `.flatpak` file |
| `--help` | Usage |

Then:

```bash
flatpak run io.github.ranrar.Marco
flatpak run io.github.ranrar.Marco.Polo
flatpak uninstall --user io.github.ranrar.Marco
```

Order does not matter, and neither script builds the other app. But to exercise
Polo's "open in Marco" handover, Marco must be **installed**, so the session bus
has something to activate.

### User installation, deliberately

`flatpak-builder` is invoked with `--user`. Its own `--help` calls that
"install dependencies in user installations", which undersells it: the same flag
decides where `--install` puts the **app**. Dropping it would silently start
installing system-wide, needing root and diverging from every other install of
these two apps. Keep it.

The distinction matters downstream too: if the same app ID ends up installed
both per-user and system-wide, `flatpak run` prefers the user copy and most
other commands become ambiguous until you pass `--user` or `--system`. That is
the usual cause of "why am I still running the old build". User data lives in
`~/.var/app/<id>/` either way, so switching install scope loses no settings.

### Versioning

The version comes from `.linux.marco` / `.linux.polo` in `build/version.json`,
and names the bundle. Nothing keeps the metainfo's `<releases>` in step with it
automatically, so the scripts **warn** when the version being built has no
matching `<release version="…">` entry — that block is what Flathub and software
centres display.

### Bundles

`flatpak build-bundle` is called with
`--runtime-repo=https://flathub.org/repo/flathub.flatpakrepo`, which embeds
where to fetch `org.gnome.Platform//50` from. Without it a bundle only installs
on machines that already have the flathub remote configured.

```bash
flatpak install ./marco_0.25.0_linux_amd64.flatpak
```

### In CI

Two things run on a runner, and they are not the same thing.

`.github/workflows/ci-flatpak.yml` is **automatic** — pull requests, and pushes
to `main`, filtered to the paths that can affect a bundle. It dry-runs both apps
as a matrix: build and finish each one into a build directory, lint it, and
check what actually landed inside. No repo, no bundle, no install, no publish.
`flatpak-builder` without `--repo` stops exactly at the point where there is
something real to inspect and nothing to ship. It builds the **local** manifests
only; the `.flathub.yml` variants build a tagged remote commit, which is by
definition not the code under review, so they are linted in the submission
layout but not built.

Its last step is the one worth knowing about: a build can succeed while quietly
shipping the wrong thing. It asserts the desktop file, metainfo, licence, icon
and all three asset directories are present, and that the D-Bus service file
matches `DBusActivatable=` — installed for Marco, absent for Polo. A partial
asset root is rejected silently at startup rather than at build time, which is
the failure this catches.

`flatpak-release-marco.yml` and `flatpak-release-polo.yml` are **manual**, one
app per workflow, **`workflow_dispatch` only** — releases are cut by hand and
nothing there tags or publishes.

| Input | Meaning |
|---|---|
| `ref` | Ref to build. Empty means the ref the run was started from |
| `manifest` | `local` (default) builds the working tree; `flathub` builds the tagged commit offline against `cargo-sources.json`, exactly as Flathub's builder will |
| `attach_to_release` + `release_tag` | Opt in to `gh release upload` onto a release that already exists |

The bundle lands as a workflow artifact either way. Three checks run before it:
`cargo-sources.json` is compared crate-by-crate against `Cargo.lock` and the run
fails if they have drifted; `manifest` mode refuses to build while the
`.flathub.yml` still carries the placeholder commit sha; and
`.github/scripts/flatpak-lint.sh` runs the linter on both the manifest and the
build directory, tolerating only `finish-args-home-filesystem-access` and
`appstream-external-screenshot-url` and failing on anything else.

`manifest: flathub` stages the manifest into a temporary directory renamed to
the app ID with `cargo-sources.json` beside it. Both are required: the linter
rejects a filename that does not match the app ID, and the manifest's
`- cargo-sources.json` source resolves relative to its own directory, one level
below where the file actually lives in this repository.

What CI cannot cover is §5's cross-launch matrix — Polo's handover to Marco
needs a session bus with an installed Marco on it. That stays a manual check.

---

## 4. Sandbox permissions

Both apps carry the same `finish-args`, plus one grant that only Polo needs:

```
--socket=wayland  --socket=fallback-x11  --share=ipc  --device=dri
--share=network        # WebView loads remote images and follows links
--filesystem=home      # see below
--talk-name=io.github.ranrar.Marco     # Polo only
```

**No font grants, on purpose.** Flatpak exposes the host's fonts and fontconfig
to the sandbox automatically; `--filesystem=xdg-config/fontconfig:ro` and
`xdg-data/fonts:ro` are redundant and the linter rejects both as unnecessary. If
the font picker looks wrong, that is a real bug, not a missing `finish-arg`.

**Marco has no `--talk-name` for Polo.** The traffic goes one way: Polo calls
Marco. Marco owns its own app ID by default, which is all
`org.freedesktop.Application` needs.

**Polo's `--talk-name` is not just for launching.** `xdg-dbus-proxy` filters
`ListActivatableNames` down to the names the app may talk to, so without the
grant an installed Marco is *invisible* to Polo and it would offer to install
something the user already has. Note this same grant would be a lint error in
the old single-Flatpak layout — `finish-args-unnecessary-appid-talk-name` fires
when the name equals the app ID. It is correct here precisely because the IDs
now differ.

### `--filesystem=home` — the one contentious permission

This produces exactly one `flatpak-builder-lint` finding on each manifest,
`finish-args-home-filesystem-access`. Expected, and not a defect: irrelevant for
a locally distributed bundle, and a Flathub submission needs an explicit
exception carrying this justification.

File **dialogs** are portal-backed and would need no static filesystem access at
all. But document-relative link and image resolution
(`marco-shared/src/logic/link_path.rs`) reads *sibling* files next to the opened
document, and the document portal grants access **per file, never per
directory** — so an image referenced as `./img/a.png` would fail to load for
every user, every time.

Tightening the *viewer's* permissions was considered at the split and rejected:
it would have eased review by breaking the feature. Polo renders the same
documents Marco does. `--filesystem=host` is the fallback if users turn out to
edit Markdown outside `$HOME`; Apostrophe, a GTK4 Markdown editor with a WebKit
preview, ships with it on Flathub, so there is precedent for either.

### Cross-launch, in one paragraph

Polo reaches Marco over D-Bus, not by spawning a sibling binary — that is what
made two sandboxes possible at all. It needs two things on Marco's side, and
neither works without the other: `DBusActivatable=true` in the desktop file, and
`io.github.ranrar.Marco.service` installed into `/app/share/dbus-1/services/`.
Flatpak does **not** generate the service file from the desktop key, but
`build-export` does check the pair — with that key set, a missing service file
is a hard error, and `Name=` must equal the app ID. Polo has neither, on
purpose. The client side is `polo/src/marco_link.rs`.

---

## 5. Verification checklist

Run 1–11 **once per app**, inside the sandbox rather than on the host. Ordered so
a failure points at what caused it.

1. **Starts at all** — proves the asset-root fix in
   `marco-shared/src/paths/platform/linux.rs` still finds `/app/share/…`. On
   failure the log shows `AssetDirMissing` listing every path searched.
2. **Themes, icons and translations load** — proves `is_valid_asset_root()`
   matched. It requires `icons/`, `themes/` **and** `language/` together; a
   partial copy is silently rejected. Each Flatpak carries its own full copy
   (~1.7 MB) — two sandboxes share nothing.
3. **Preview renders** — the WebKit web process started. If it crashes, capture
   `flatpak run --log-session-bus <id>` and check for a nested-sandbox or a11y
   name rejection.
4. **Open a file from the dialog**, then one from the command line
   (`flatpak run <id> path/to/x.md`).
5. **Document-relative images resolve** — the reason for the filesystem grant.
   Open a document referencing `./img/foo.png`.
6. **Recent files survive a restart** — the list stores absolute paths
   (`marco-shared/src/paths/polo.rs`), and a file arriving through the document
   portal has an unstable `/run/user/<uid>/doc/<hash>/…` path.
7. **Config persists** under `~/.var/app/<id>/config/` and portable mode did not
   trigger.
8. **System dark/light switch is followed live** — portal Settings path.
9. **Font picker lists user fonts** — see the note above; no grant exists for it.
10. **Correct name and icon** in the shell, the taskbar and GNOME Resources.
11. **`flatpak-builder-lint manifest <path>` is clean** apart from
    `finish-args-home-filesystem-access`.

Then, across the two:

- **Install Marco only** → launches, opens files, correct name and icon.
- **Install Polo only** → launches; "open in Marco" shows the **Install Marco**
  prompt, and that prompt opens the store page.
- **Install both** → both cross-launch buttons work; DualView closes Polo,
  Editor-and-View-Separate does not.
- **Install Marco while Polo runs** → the button switches from install to open,
  without restarting Polo.
- **Uninstall Marco while Polo runs** → Polo falls back to the install prompt
  rather than erroring.
- **GNOME Resources shows "Marco" and "Polo" as separate, correctly named
  entries** — the defect the split exists to fix.

---

## 6. Flathub

Not submitted yet. Two submissions, Marco first — a second app referring to an
accepted first one is an easier conversation.

What follows here is the vendoring recipe and a summary. The full A-to-Z — the
pre-flight checklist, the submission itself, what acceptance does, releasing
updates, and how much of it can be automated — is kept as a working note outside
the repository.

### Vendoring — required for Flathub, not for local builds

Flathub builders have no network, so every crate must be a checksummed source
entry. Local builds skip all of this: the `.yml` manifests grant
`--share=network` in `build-args` and let cargo fetch. The `.flathub.yml`
variants do not, and use `cargo --offline` plus a git source pinned to a tag and
full commit sha.

```bash
git clone --depth 1 https://github.com/flatpak/flatpak-builder-tools.git \
    ~/.local/src/flatpak-builder-tools

# PEP 668 is active on this machine, so `pip install --user` is refused.
python3 -m venv ~/.local/venv/flatpak-cargo
~/.local/venv/flatpak-cargo/bin/pip install aiohttp tomlkit

# From the workspace root
~/.local/venv/flatpak-cargo/bin/python \
    ~/.local/src/flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
    ./Cargo.lock -o build/linux/flatpak/cargo-sources.json
```

Only `aiohttp` and `tomlkit` are needed — older guides say `toml` and `siphash`;
the current generator uses neither, and `python3-siphash` has no apt candidate
anyway, which is a good sign you are reading stale instructions.

The result is ~1145 source entries (~318 KB) and takes about three seconds: it
reads URLs and hashes straight out of `Cargo.lock` rather than downloading, the
one exception being git dependencies, which it must clone. **Regenerate every
time `Cargo.lock` changes, and commit it.** One `cargo-sources.json` serves both
manifests — it is generated from the workspace lockfile, which covers both
binaries.

Two workspace specifics, both settled:

- **`wry` is a git dependency on a fork**, pinned to a rev. It vendored cleanly;
  the generator emits a `git` source plus a shell command copying it into
  `cargo/vendor/wry`. The known breakage in this area is limited to *cyclic* git
  dependency graphs (a cargo bug); a single pinned rev is unaffected. A reviewer
  will still ask why a fork of a WebView library is needed — the answer is the
  `gtk4-webkit6` branch.
- **`marco-core` comes from crates.io** and vendors normally.

The generator's `dest` is `cargo`, which is why the manifests set
`CARGO_HOME: /run/build/<app>/cargo` — cargo must find the emitted
`cargo/config` that redirects crates.io to `cargo/vendor`. Change one and you
must change the other.

### Submission

1. Confirm both metainfo files pass `appstreamcli validate --strict`. The one
   expected hint is `cid-contains-uppercase-letter`, on both — it must **not**
   be "fixed": lowercasing breaks Flathub's ownership check.
2. Tag a release and fill `tag` + full commit sha into both `.flathub.yml`.
3. PR against `github.com/flathub/flathub` on the `new-pr` branch, adding the
   manifest plus `cargo-sources.json`.
4. Request the `--filesystem=home` exception for **both** apps, with the
   justification in §4. Expect that conversation twice.
5. Expect questions about the `wry` fork and the five-component Polo ID.
6. Once accepted, Flathub builds from its own repo — so the release process
   needs a step that regenerates `cargo-sources.json` and opens a PR there on
   each version bump.

---

## 7. Known gaps

| Item | Impact | Where |
|---|---|---|
| Releases are cut by hand | `release.yml` was removed deliberately. Nothing tags, and nothing publishes a GitHub release — `flatpak-release-marco.yml` and `flatpak-release-polo.yml` only build and verify, on manual dispatch | `.github/workflows/` |
| `xdg-open` hands the host a sandbox-only path | The log-folder button in settings does nothing under Flatpak; use `UriLauncher` | `marco/src/ui/settings/tabs/debug.rs` |
| Recent-files entries rot for portal-opened files | Absolute `/run/user/<uid>/doc/…` paths are not stable across sessions | `marco-shared/src/paths/polo.rs` |
| `marco-shared/build.rs` copies `fonts` and `documentation`, which do not exist, and never copies `img/` | Pre-existing, not Flatpak-specific; anything rendering from `assets/img/` is already broken in installed builds | `marco-shared/build.rs`, `build_deb.sh` |

---

## 8. Sources

- [org.freedesktop.Sdk.Extension.rust-stable, branch 25.08](https://github.com/flathub/org.freedesktop.Sdk.Extension.rust-stable/blob/branch/25.08/org.freedesktop.Sdk.Extension.rust-stable.json)
- [Flathub: MetaInfo guidelines](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines)
- [Flathub: submission requirements](https://docs.flathub.org/docs/for-app-authors/requirements)
- [Flathub: the linter](https://docs.flathub.org/linter)
- [Flatpak: requirements & conventions](https://docs.flatpak.org/en/latest/conventions.html)
- [Flatpak: sandbox permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html)
- [flatpak-cargo-generator.py](https://github.com/flatpak/flatpak-builder-tools/blob/master/cargo/flatpak-cargo-generator.py)
- [Git dependencies in Rust Flatpaks (cyclic-graph caveat)](https://discourse.flathub.org/t/getting-errors-with-building-rust-flatpaks-that-depend-on-git-dependencies/10000)
- [Apostrophe's Flathub manifest](https://github.com/flathub/org.gnome.gitlab.somas.Apostrophe/blob/master/org.gnome.gitlab.somas.Apostrophe.json) — GTK4 Markdown editor precedent
