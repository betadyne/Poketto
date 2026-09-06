#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob

export PATH="$HOME/.cargo/bin:$PATH"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DRY_RUN=0
SKIP_UPLOAD=0
ONLY="all"
CLEANUP=0

usage() {
  cat <<EOF
Usage: scripts/release-local.sh [OPTIONS]

Build Poketto release artifacts for Linux and Windows (x64),
rename them to Poketto-<version>-<os>-<arch>.<ext> in dist-release/,
and upload them to a GitHub Release.

Options:
  --dry-run        Fake the compiler/bundler outputs, run the real
                   collect/rename/package steps, skip the upload.
  --skip-upload    Real builds, but do not create tag or GitHub Release.
  --only <target>  linux | windows (default: build both).
  --cleanup        Remove dist-release/ and src-tauri/target after a
                   successful upload.
  -h, --help       Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --skip-upload) SKIP_UPLOAD=1 ;;
    --only)
      ONLY="${2:-}"
      [[ "$ONLY" == "linux" || "$ONLY" == "windows" ]] || { echo "error: --only accepts linux|windows" >&2; exit 1; }
      shift
      ;;
    --cleanup) CLEANUP=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "error: unknown option $1" >&2; usage >&2; exit 1 ;;
  esac
  shift
done

log() { printf '[release] %s\n' "$*"; }
warn() { printf '[release] WARNING: %s\n' "$*" >&2; }
die() { printf '[release] ERROR: %s\n' "$*" >&2; exit 1; }

VERSION="$(sed -n 's/^[[:space:]]*"version":[[:space:]]*"\([^"]*\)".*/\1/p' package.json | head -n 1)"
[[ -n "$VERSION" ]] || die "no version found in package.json"

OUT="dist-release"
BASE="Poketto-${VERSION}"
BUNDLE="src-tauri/target/release/bundle"
WIN_BUNDLE="src-tauri/target/x86_64-pc-windows-gnu/release/bundle"
LINUX_BIN="src-tauri/target/release/poketto"
WIN_BIN="src-tauri/target/x86_64-pc-windows-gnu/release/poketto.exe"
SIGN_ARGS=()
if [[ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" && -f "$HOME/.tauri/poketto.key" ]]; then
  export TAURI_SIGNING_PRIVATE_KEY="$(cat "$HOME/.tauri/poketto.key")"
  export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}"
  log "using signing key $HOME/.tauri/poketto.key"
fi
if [[ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]]; then
  SIGN_ARGS+=(--no-sign)
fi

if [[ "$DRY_RUN" == "1" ]]; then
  SKIP_UPLOAD=1
  FAKE="$(mktemp -d)"
  trap 'rm -rf "$FAKE"' EXIT
  BUNDLE="$FAKE/bundle"
  WIN_BUNDLE="$FAKE/win-bundle"
  LINUX_BIN="$FAKE/poketto"
  WIN_BIN="$FAKE/poketto.exe"
  mkdir -p "$BUNDLE/appimage" "$BUNDLE/deb" "$BUNDLE/rpm" "$WIN_BUNDLE/nsis"
  echo fake > "$BUNDLE/appimage/poketto_${VERSION}_amd64.AppImage"
  echo fake > "$BUNDLE/deb/poketto_${VERSION}_amd64.deb"
  echo fake > "$BUNDLE/rpm/poketto-${VERSION}-1.x86_64.rpm"
  echo fake > "$WIN_BUNDLE/nsis/Poketto_${VERSION}_x64-setup.exe"
  echo fake > "$LINUX_BIN"
  echo fake > "$WIN_BIN"
  log "dry-run mode: compiler/bundler outputs are faked"
fi

collect_first() {
  local pattern="$1" dest="$2" label="$3"
  local matches=( $pattern )
  if [[ "${#matches[@]}" == "0" ]]; then
    warn "$label not found ($pattern); skipping"
    return 1
  fi
  [[ "${#matches[@]}" == "1" ]] || warn "$label matched ${#matches[@]} files; using ${matches[0]}"
  cp "${matches[0]}" "$dest"
  log "$label -> $(basename "$dest")"
}

build_linux() {
  log "building Linux bundles (appimage, deb, rpm)"
  if [[ "$DRY_RUN" == "1" ]]; then
    log "dry-run: skipping npm run tauri build"
  else
    rm -rf "$BUNDLE"
    NO_STRIP=1 npm run tauri build -- --bundles appimage,deb,rpm "${SIGN_ARGS[@]}"
  fi
  collect_first "$BUNDLE/appimage/*.AppImage" "$OUT/${BASE}-linux-x64.AppImage" "AppImage" || true
  collect_first "$BUNDLE/deb/*.deb" "$OUT/${BASE}-linux-x64.deb" "deb" || true
  collect_first "$BUNDLE/rpm/*.rpm" "$OUT/${BASE}-linux-x64.rpm" "rpm" || true

  log "packing portable tar.xz"
  if [[ -f "$LINUX_BIN" ]]; then
    local stage="$OUT/${BASE}-linux-x64"
    rm -rf "$stage"
    mkdir -p "$stage"
    cp "$LINUX_BIN" "$stage/poketto"
    chmod +x "$stage/poketto"
    if [[ -f "src-tauri/icons/icon.png" ]]; then
      cp "src-tauri/icons/icon.png" "$stage/poketto.png"
    else
      warn "src-tauri/icons/icon.png missing; portable bundle ships without icon"
    fi
    cat > "$stage/poketto.desktop" <<EOF
[Desktop Entry]
Name=Poketto
Comment=Visual Novel Game Launcher
Exec=poketto
Icon=poketto
Type=Application
Categories=Game;
Terminal=false
StartupWMClass=poketto
EOF
    tar -cJf "$OUT/${BASE}-linux-x64.tar.xz" -C "$OUT" "${BASE}-linux-x64"
    rm -rf "$stage"
    log "portable -> ${BASE}-linux-x64.tar.xz"
  else
    warn "Linux binary not found ($LINUX_BIN); skipping tar.xz"
  fi
}

have_windows_target() {
  rustup target list --installed 2>/dev/null | grep -q "x86_64-pc-windows-gnu"
}

have_windows_linker() {
  [[ -n "${CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER:-}" ]] || command -v x86_64-w64-mingw32-gcc >/dev/null
}

build_windows() {
  log "building Windows x64 artifacts"
  if [[ "$DRY_RUN" == "0" ]]; then
    if ! have_windows_target || ! have_windows_linker; then
      warn "Windows cross toolchain incomplete; skipping Windows artifacts (Linux artifacts are kept)"
      warn "to enable: rustup target add x86_64-pc-windows-gnu && sudo dnf install mingw64-gcc"
      return 0
    fi
    if [[ -z "${CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER:-}" ]] && command -v x86_64-w64-mingw32-gcc >/dev/null; then
      export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="x86_64-w64-mingw32-gcc"
    fi
    rm -rf "$WIN_BUNDLE"
    npm run tauri build -- --target x86_64-pc-windows-gnu --bundles nsis "${SIGN_ARGS[@]}" || \
      warn "NSIS bundle failed (tauri auto-downloads NSIS on first run; retry or install nsis); continuing"
  else
    log "dry-run: skipping npm run tauri build --target x86_64-pc-windows-gnu"
  fi
  collect_first "$WIN_BUNDLE/nsis/*.exe" "$OUT/${BASE}-windows-x64.exe" "NSIS setup" || true

  if [[ ! -f "$WIN_BIN" && "$DRY_RUN" == "0" ]]; then
    cargo build --release --target x86_64-pc-windows-gnu --manifest-path src-tauri/Cargo.toml || \
      warn "Windows cross compile failed; skipping portable zip"
  fi
  if [[ -f "$WIN_BIN" ]]; then
    (cd "$(dirname "$WIN_BIN")" && zip -j "$ROOT/$OUT/${BASE}-windows-x64.zip" "$(basename "$WIN_BIN")") >/dev/null || \
      warn "zip failed; skipping portable zip"
    [[ -f "$OUT/${BASE}-windows-x64.zip" ]] && log "portable -> ${BASE}-windows-x64.zip"
  else
    warn "Windows binary not found ($WIN_BIN); skipping portable zip"
  fi
}

upload_release() {
  if [[ "$SKIP_UPLOAD" == "1" ]]; then
    log "upload skipped (--dry-run/--skip-upload)"
    return 0
  fi
  gh auth status >/dev/null || die "gh is not authenticated (run: gh auth login)"
  local tag="v${VERSION}"
  if ! git rev-parse "$tag" >/dev/null 2>&1; then
    git tag "$tag"
    log "created tag $tag"
  fi
  git push origin "$tag"
  if gh release view "$tag" >/dev/null 2>&1; then
    gh release upload "$tag" "$OUT"/* --clobber
  else
    # shellcheck disable=SC2086
    gh release create "$tag" "$OUT"/* --title "Poketto v${VERSION}" --generate-notes
  fi
  ensure_release_notes "$tag"
  log "uploaded dist-release/* to $tag"
}
ensure_release_notes() {
  local tag="$1" marker="Recommended downloads:" body header="${IMPORTANT_NOTES:-}"
  body="$(gh release view "$tag" --json body -q .body)"
  [[ "$body" == *"$marker"* ]] && return 0
  {
    [[ -n "$header" ]] && printf '%s\n\n' "$header"
    printf '%s\n\n---\n**%s** Linux users who want in-app auto-update should grab the `.AppImage`. System packages (`.deb`, `.rpm`) and portable archives update manually from this page.\n' "$body" "$marker"
  } | gh release edit "$tag" --notes-file -
  log "added download recommendation to $tag notes"
}

publish_updater_json() {
  [[ "$DRY_RUN" == "1" || "${#SIGN_ARGS[@]}" -gt 0 ]] && return 0
  local base_url="https://github.com/betadyne/Poketto/releases/download/v${VERSION}"
  local entries=()
  local appimage=($BUNDLE/appimage/*.AppImage)
  local setup=($WIN_BUNDLE/nsis/*-setup.exe)
  [[ "${#appimage[@]}" == "1" && -f "${appimage[0]}.sig" ]] && \
    entries+=("linux-x86_64|${appimage[0]}|${base_url}/${BASE}-linux-x64.AppImage")
  [[ "${#setup[@]}" == "1" && -f "${setup[0]}.sig" ]] && \
    entries+=("windows-x86_64|${setup[0]}|${base_url}/${BASE}-windows-x64.exe")
  if [[ "${#entries[@]}" == "0" ]]; then
    warn "no signed updater bundles found; in-app updates will stay silent"
    return 0
  fi
  UPDATER_VERSION="$VERSION" UPDATER_NOTES="${IMPORTANT_NOTES:-}" \
    python3 - "$OUT/latest.json" "${entries[@]}" <<'PYEOF'
import datetime, json, os, sys
platforms = {}
for arg in sys.argv[2:]:
    key, path, url = arg.split("|", 2)
    with open(path + ".sig") as handle:
        platforms[key] = {"signature": handle.read().strip(), "url": url}
manifest = {
    "version": os.environ["UPDATER_VERSION"],
    "notes": os.environ.get("UPDATER_NOTES", ""),
    "pub_date": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
    "platforms": platforms,
}
with open(sys.argv[1], "w") as handle:
    json.dump(manifest, handle, indent=2)
PYEOF
  log "updater -> latest.json with ${#entries[@]} platform(s)"
}

rm -rf "$OUT"
mkdir -p "$OUT"
if [[ "$DRY_RUN" == "0" && "${#SIGN_ARGS[@]}" -gt 0 ]]; then
  warn "TAURI_SIGNING_PRIVATE_KEY is unset; bundles will be unsigned and updater signatures unavailable"
fi

[[ "$ONLY" == "all" || "$ONLY" == "linux" ]] && build_linux
[[ "$ONLY" == "all" || "$ONLY" == "windows" ]] && build_windows

publish_updater_json

log "dist-release contents:"
ls -la "$OUT"

upload_release

if [[ "$CLEANUP" == "1" && "$SKIP_UPLOAD" == "0" ]]; then
  rm -rf "$OUT" src-tauri/target
  log "cleaned dist-release and src-tauri/target"
fi
