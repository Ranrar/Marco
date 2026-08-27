# Shared implementation behind build_flatpak_marco.sh and build_flatpak_polo.sh.
#
# Not executable on its own -- source it from an app script that has already set
# the variables listed under "CONTRACT" below, then call `flatpak_main "$@"`.
#
# WHY THIS FILE EXISTS
#   Marco and Polo are two Flatpaks with two manifests, but the machinery around
#   them is identical: dependency checks, version reading, flatpak-builder
#   invocation, linting, bundling, install. Two standalone scripts would mean
#   two copies of ~250 lines that drift apart the first time one is fixed and
#   the other is not. The per-app scripts stay short enough to read in one go.
#
# CONTRACT -- the sourcing script must set all of these before calling
# flatpak_main:
#
#   APP_NAME        Human name for output, e.g. "Marco"
#   APP_ROLE        One-word role for output, e.g. "editor"
#   APP_ID          Flatpak app ID, e.g. io.github.ranrar.Marco
#   APP_SUBDIR      Directory under build/linux/flatpak/ holding the manifest
#   APP_COMMAND     The `command:` in the manifest, e.g. markdowncomposer
#   VERSION_KEY     Key under .linux in build/version.json, e.g. marco
#   BUNDLE_PREFIX   Leading part of the bundle filename, e.g. marco
#   REQUIRED_FILES  Array of paths relative to the app subdirectory

set -euo pipefail

umask 022

# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo ""
}

print_success() { echo -e "${GREEN}OK: $1${NC}"; }
print_error() { echo -e "${RED}ERROR: $1${NC}"; }
print_warning() { echo -e "${YELLOW}WARN: $1${NC}"; }
print_info() { echo -e "${BLUE}INFO: $1${NC}"; }

# ---------------------------------------------------------------------------
# Paths and settings
# ---------------------------------------------------------------------------

# Runtime the manifests target. Keep in sync with `runtime-version` in both;
# --check verifies these are actually installed.
RUNTIME_VERSION="50"
RUST_EXT_VERSION="25.08"

# Where to fetch the runtime from when installing a bundle on a machine that has
# no flathub remote. Without this a bundle is only installable where
# org.gnome.Platform is already present.
RUNTIME_REPO="https://flathub.org/repo/flathub.flatpakrepo"

DO_INSTALL="true"
DO_BUNDLE="true"
CHECK_ONLY="false"

flatpak_init_paths() {
    # This file lives in build/linux/flatpak/, so the workspace root is three
    # levels up from it.
    COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    ROOT_DIR="$(cd "$COMMON_DIR/../../.." && pwd)"
    cd "$ROOT_DIR"

    # The directory name this script and the manifests live under, relative to
    # the workspace root. The manifests' `install` commands hardcode it, so a
    # rename must be caught here rather than inside the sandbox.
    FLATPAK_DIR_NAME="$(basename "$COMMON_DIR")"
    FLATPAK_DIR="build/linux/${FLATPAK_DIR_NAME}"
    APP_DIR="${FLATPAK_DIR}/${APP_SUBDIR}"

    MANIFEST="${APP_DIR}/${APP_ID}.yml"
    METAINFO="${APP_DIR}/${APP_ID}.metainfo.xml"

    # Per-app build and repo directories, so the two apps never trample each
    # other. Both sit under _build/ and _repo/, which the manifests' `skip`
    # lists already exclude from the source copy.
    BUILD_DIR="${FLATPAK_DIR}/_build/${APP_SUBDIR}"
    REPO_DIR="${FLATPAK_DIR}/_repo/${APP_SUBDIR}"

    INSTALLER_DIR="$ROOT_DIR/build/installer"
    VERSION_FILE="$ROOT_DIR/build/version.json"
}

# ---------------------------------------------------------------------------
# Arguments
# ---------------------------------------------------------------------------

flatpak_show_help() {
    cat << EOF
Build a Flatpak bundle for ${APP_NAME} (${APP_ROLE}).

${APP_NAME} and its companion are SEPARATE Flatpaks with separate manifests.
This script builds only ${APP_NAME} (${APP_ID}); the other has its own script in
this directory.

USAGE
    bash ${FLATPAK_DIR}/$(basename "$0") [OPTIONS]

OPTIONS
    --check         Verify build dependencies and exit
    --no-install    Build, but do not install into the user's flatpak
    --no-bundle     Build (and install), but do not emit a .flatpak bundle
    --help          Show this help

OUTPUT
    ${BUILD_DIR}/     flatpak-builder working directory
    ${REPO_DIR}/      OSTree repository
    build/installer/${BUNDLE_PREFIX}_VERSION_linux_amd64.flatpak

NOTES
    This builds for LOCAL distribution: cargo is given network access at build
    time. A Flathub submission needs offline vendoring instead -- see
    ${FLATPAK_DIR}/README.md and ${APP_DIR}/${APP_ID}.flathub.yml.
EOF
}

flatpak_parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --check) CHECK_ONLY="true"; shift ;;
            --no-install) DO_INSTALL="false"; shift ;;
            --no-bundle) DO_BUNDLE="false"; shift ;;
            --help|-h) flatpak_show_help; exit 0 ;;
            *)
                print_error "Unknown option: $1"
                echo "Use 'bash ${FLATPAK_DIR}/$(basename "$0") --help' for usage information"
                exit 1
                ;;
        esac
    done
}

# ---------------------------------------------------------------------------
# Dependency checks
# ---------------------------------------------------------------------------

flatpak_check_dependencies() {
    print_header "Checking Dependencies for ${APP_NAME}"

    local missing="false"

    if command -v flatpak &>/dev/null; then
        print_success "flatpak found ($(flatpak --version))"
    else
        print_error "flatpak not found. Install it with: sudo apt install flatpak"
        missing="true"
    fi

    if command -v python3 &>/dev/null; then
        print_success "python3 found ($(python3 --version))"
    else
        print_error "python3 not found (needed to read build/version.json)"
        missing="true"
    fi

    # flatpak-builder is used as a flatpak (org.flatpak.Builder); a host
    # package of the same name also works.
    if flatpak info org.flatpak.Builder &>/dev/null; then
        print_success "org.flatpak.Builder found"
    elif command -v flatpak-builder &>/dev/null; then
        print_success "host flatpak-builder found ($(flatpak-builder --version))"
    else
        print_error "flatpak-builder not found. Install it with:"
        echo "    flatpak install -y flathub org.flatpak.Builder"
        missing="true"
    fi

    local -a refs=(
        "org.gnome.Platform//${RUNTIME_VERSION}"
        "org.gnome.Sdk//${RUNTIME_VERSION}"
        "org.freedesktop.Sdk.Extension.rust-stable//${RUST_EXT_VERSION}"
    )
    local ref
    for ref in "${refs[@]}"; do
        if flatpak info "$ref" &>/dev/null; then
            print_success "$ref found"
        else
            print_error "$ref not installed. Install it with:"
            echo "    flatpak install -y flathub $ref"
            missing="true"
        fi
    done

    if [ -f "$MANIFEST" ]; then
        print_success "Manifest found: $MANIFEST"
    else
        print_error "Manifest not found: $MANIFEST"
        missing="true"
    fi

    # The manifest hardcodes this app's directory in its install commands.
    if grep -q "build/linux/${FLATPAK_DIR_NAME}/${APP_SUBDIR}/" "$MANIFEST" 2>/dev/null; then
        print_success "Manifest paths match '${FLATPAK_DIR_NAME}/${APP_SUBDIR}'"
    else
        print_error "Manifest does not reference 'build/linux/${FLATPAK_DIR_NAME}/${APP_SUBDIR}/'."
        echo "    This directory appears to have been renamed. Update the"
        echo "    'install -Dm644 build/linux/.../' paths in:"
        echo "      $MANIFEST"
        missing="true"
    fi

    local f
    for f in "${REQUIRED_FILES[@]}"; do
        if [ -f "${APP_DIR}/${f}" ]; then
            print_success "$(basename "$f") present"
        else
            print_error "Missing required file: ${APP_DIR}/${f}"
            missing="true"
        fi
    done

    if [ "$missing" = "true" ]; then
        print_error "Missing dependencies. See messages above."
        exit 1
    fi

    print_success "All required dependencies found!"
}

# ---------------------------------------------------------------------------
# Versioning
# ---------------------------------------------------------------------------

flatpak_read_version() {
    print_header "Versioning"

    if [ ! -f "$VERSION_FILE" ]; then
        print_error "Version file not found: $VERSION_FILE"
        exit 1
    fi

    VERSION="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["linux"][sys.argv[2]])' \
        "$VERSION_FILE" "$VERSION_KEY")"

    if [ -z "$VERSION" ]; then
        print_error "Could not read .linux.${VERSION_KEY} from $VERSION_FILE"
        exit 1
    fi

    print_info "Version: $VERSION"

    # The metainfo's <releases> drives what Flathub and software centres
    # display. Nothing keeps it in step with version.json automatically, so warn
    # on drift.
    if ! grep -q "version=\"${VERSION}\"" "$METAINFO"; then
        print_warning "metainfo.xml has no <release version=\"${VERSION}\"> entry."
        print_warning "Add one to $METAINFO or the listed version will lag."
    fi
}

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

flatpak_build() {
    print_header "Building ${APP_NAME}"

    # ostree creates the repo with a single mkdirat() and fails outright if the
    # PARENT directory is missing:
    #   error: Creating repo: mkdirat: No such file or directory
    # Since the split, both apps build into subdirectories (_build/<app>,
    # _repo/<app>), and _build/ and _repo/ are gitignored -- so on a clean
    # checkout, or after either is deleted, those parents do not exist. Create
    # them, and let flatpak-builder own the leaf build directory it --force-cleans.
    mkdir -p "$(dirname "$BUILD_DIR")" "$(dirname "$REPO_DIR")"

    if flatpak info org.flatpak.Builder &>/dev/null; then
        FB=(flatpak run org.flatpak.Builder)
        LINT=(flatpak run --command=flatpak-builder-lint org.flatpak.Builder)
    else
        FB=(flatpak-builder)
        LINT=()
    fi

    local -a builder_args=(
        --force-clean

        # Do not remove this. flatpak-builder's own help calls --user "Install
        # dependencies in user installations", which undersells it: the same
        # flag decides where --install puts the APP. Its do_install() emits
        # `flatpak install --user` when set and `flatpak install --system` when
        # not, and --system is the default. Dropping --user would silently start
        # installing system-wide, needing root and diverging from every other
        # install of these two apps.
        --user

        --repo="$REPO_DIR"

        # rofiles-fuse is a hardlink-safety layer flatpak-builder mounts over
        # the build tree. It needs a working FUSE, which is unavailable in
        # containers and some sandboxed shells -- there it fails with
        #   "fusermount: file descriptor 4 is not a socket"
        #   "Error: Failure spawning rofiles-fuse, exit_status: 256"
        # Disabling it only removes a guard against a build writing back into
        # its own cached sources; the build output is identical.
        --disable-rofiles-fuse
    )

    if [ "$DO_INSTALL" = "true" ]; then
        builder_args+=(--install)
    fi

    print_info "Running flatpak-builder (this compiles ${APP_NAME} in release mode)"
    "${FB[@]}" "${builder_args[@]}" "$BUILD_DIR" "$MANIFEST"

    print_success "${APP_NAME} built"
}

# ---------------------------------------------------------------------------
# Lint (advisory)
# ---------------------------------------------------------------------------

flatpak_lint() {
    [ "${#LINT[@]}" -gt 0 ] || return 0

    print_header "Linting ${APP_NAME}"

    # Advisory only: flatpak-builder-lint enforces Flathub's rules, which are
    # stricter than what a locally distributed bundle needs. Both apps are
    # expected to report finish-args-home-filesystem-access; that grant stays,
    # because document-relative images cannot be served through a portal.
    if "${LINT[@]}" manifest "$MANIFEST"; then
        print_success "Manifest lint clean"
    else
        print_warning "Manifest lint reported findings (advisory for local builds)"
    fi

    if "${LINT[@]}" repo "$REPO_DIR"; then
        print_success "Repo lint clean"
    else
        print_warning "Repo lint reported findings (advisory for local builds)"
    fi
}

# ---------------------------------------------------------------------------
# Bundle
# ---------------------------------------------------------------------------

flatpak_bundle() {
    [ "$DO_BUNDLE" = "true" ] || return 0

    print_header "Creating Bundle"

    mkdir -p "$INSTALLER_DIR"
    BUNDLE="$INSTALLER_DIR/${BUNDLE_PREFIX}_${VERSION}_linux_amd64.flatpak"

    # --runtime-repo embeds where to get org.gnome.Platform from, so the bundle
    # installs on a machine with no flathub remote. Without it the user has to
    # add flathub by hand first.
    flatpak build-bundle --runtime-repo="$RUNTIME_REPO" "$REPO_DIR" "$BUNDLE" "$APP_ID"

    print_success "Bundle created: $BUNDLE"
    print_info "Size: $(du -h "$BUNDLE" | cut -f1)"
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

flatpak_summary() {
    print_header "Build Complete"

    echo "${APP_NAME} built successfully!"
    echo ""

    if [ "$DO_INSTALL" = "true" ]; then
        print_success "To run it:"
        echo "  flatpak run ${APP_ID}"
        echo ""
        print_success "To uninstall:"
        echo "  flatpak uninstall --user ${APP_ID}"
        echo ""
    fi

    if [ "$DO_BUNDLE" = "true" ]; then
        print_success "To install the bundle on another machine:"
        echo "  flatpak install ./${BUNDLE_PREFIX}_${VERSION}_linux_amd64.flatpak"
        echo ""
        print_info "The bundle carries the flathub runtime-repo, so the"
        print_info "org.gnome.Platform//${RUNTIME_VERSION} runtime can be pulled in"
        print_info "even without the flathub remote already configured."
    fi
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

flatpak_main() {
    flatpak_init_paths
    flatpak_parse_args "$@"
    flatpak_check_dependencies

    if [ "$CHECK_ONLY" = "true" ]; then
        exit 0
    fi

    flatpak_read_version
    flatpak_build
    flatpak_lint
    flatpak_bundle
    flatpak_summary
}
