#!/bin/bash
# Run flatpak-builder-lint and fail only on findings that are actually defects.
#
#   bash .github/scripts/flatpak-lint.sh manifest path/to/io.github.ranrar.Marco.yml
#   bash .github/scripts/flatpak-lint.sh builddir path/to/build
#
# Shared by release-marco.yml and release-polo.yml. Both apps carry the same
# permissions, so both produce the same tolerated findings.
#
# TOLERATED, and why:
#
#   finish-args-home-filesystem-access
#       --filesystem=home. Document-relative links and images resolve against
#       the opened file's directory, and the document portal grants access per
#       file, never per directory. This is the finding the Flathub exception
#       request covers; the justification is in build/linux/flatpak/README.md
#       section 4. It is not a defect and must not be "fixed" by dropping the
#       grant -- that breaks relative images for every user.
#
#   appstream-external-screenshot-url  (builddir only)
#       Screenshots are not mirrored to dl.flathub.org yet. Flathub does that
#       mirroring itself, after acceptance. Nothing to do upstream.
#
# Anything else fails the run.

set -euo pipefail

mode="${1:?usage: flatpak-lint.sh <manifest|builddir> <path>}"
target="${2:?usage: flatpak-lint.sh <manifest|builddir> <path>}"

report="$(mktemp)"

# The target's own directory is granted explicitly on top of --filesystem=host:
# `host` covers the real filesystem but NOT /tmp, which is private to the
# sandbox, so a manifest staged under /tmp would be invisible to the linter and
# fail as "no such manifest file".
target_dir="$(cd "$(dirname "$target")" && pwd)"

# The linter exits non-zero whenever it reports anything, including the findings
# above, so its status is not the signal -- the JSON is.
flatpak run --filesystem=host --filesystem="$target_dir" \
    --command=flatpak-builder-lint \
    org.flatpak.Builder "$mode" "$target" >"$report" || true

echo "--- flatpak-builder-lint $mode $target"
cat "$report"
echo "---"

MODE="$mode" python3 - "$report" <<'PY'
import json, os, sys

TOLERATED = {"finish-args-home-filesystem-access"}
if os.environ["MODE"] == "builddir":
    TOLERATED.add("appstream-external-screenshot-url")

try:
    report = json.load(open(sys.argv[1]))
except json.JSONDecodeError:
    print("::error::flatpak-builder-lint produced no parsable report.")
    sys.exit(1)

unexpected = sorted(set(report.get("errors", [])) - TOLERATED)
if unexpected:
    for finding in unexpected:
        print(f"::error::flatpak-builder-lint: {finding}")
    print("See https://docs.flathub.org/linter for details and exceptions.")
    sys.exit(1)

tolerated = sorted(set(report.get("errors", [])) & TOLERATED)
if tolerated:
    print("Tolerated findings (expected, documented):", ", ".join(tolerated))
print("No unexpected findings.")
PY
