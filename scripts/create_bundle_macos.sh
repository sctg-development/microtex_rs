#!/usr/bin/env bash
set -euo pipefail

# Creates a dependency bundle suitable for committing under
# dependencies_bundle/macos/intel. Overwrites DEST if it exists.

DEST=${1:-dependencies_bundle/macos/intel}

echo "Preparing bundle at: $DEST"
rm -rf "$DEST"
mkdir -p "$DEST/lib/pkgconfig" "$DEST/include" "$DEST/share/licenses" || true

# Packages we want to capture from Homebrew pkg-config
PACKAGES=(cairo pango fontconfig freetype harfbuzz pixman pcre2 libpng lzo)

# Helper: copy files preserving structure
copy_libs() {
  local libdir="$1"
  if [ -d "$libdir" ]; then
    echo "Copying libs from $libdir"
    shopt -s nullglob
    for f in "$libdir"/*.{dylib,a,so}; do
      cp -a "$f" "$DEST/lib/" || true
    done
    shopt -u nullglob

    if [ -d "$libdir/pkgconfig" ]; then
      echo "Copying pkgconfig files from $libdir/pkgconfig"
      cp -a "$libdir/pkgconfig"/*.pc "$DEST/lib/pkgconfig/" 2>/dev/null || true
    fi
  fi
}

copy_includes() {
  local incdir="$1"
  if [ -d "$incdir" ]; then
    echo "Copying headers from $incdir"
    rsync -a --exclude='*.la' "$incdir/" "$DEST/include/" || true
  fi
}

# iterate packages
for pkg in "${PACKAGES[@]}"; do
  echo "Processing package: $pkg"
  if ! pkg-config --exists "$pkg"; then
    echo "pkg-config: $pkg not found; please brew install $pkg" >&2
    exit 1
  fi

  libdir=$(pkg-config --variable=libdir "$pkg" 2>/dev/null || true)
  incdir=$(pkg-config --variable=includedir "$pkg" 2>/dev/null || true)

  # Fallbacks for common Homebrew layouts
  if [ -z "$libdir" ]; then
    if [ -d "/usr/local/lib" ]; then libdir="/usr/local/lib"; fi
    if [ -d "/opt/homebrew/lib" ]; then libdir="/opt/homebrew/lib"; fi
  fi
  if [ -z "$incdir" ]; then
    if [ -d "/usr/local/include" ]; then incdir="/usr/local/include"; fi
    if [ -d "/opt/homebrew/include" ]; then incdir="/opt/homebrew/include"; fi
  fi

  copy_libs "$libdir"
  copy_includes "$incdir"

  # Try to copy license files if available nearby
  if [ -n "$libdir" ]; then
    for lic in "$libdir"/../share/licenses/$pkg/*; do
      [ -e "$lic" ] && cp -a "$lic" "$DEST/share/licenses/" 2>/dev/null || true
    done
  fi
done

# Add a small README to the bundle
cat > "$DEST/README.md" <<'EOF'
This directory is a generated dependency bundle for macOS Intel.

It contains:
- lib/  -> .dylib and .a
- lib/pkgconfig/ -> .pc files
- include/ -> headers

To use it: set MICROTEX_BUNDLE_DIR to the path of this bundle before building.
EOF

# Create an index of copied files (helpful to review what was included)
find "$DEST" -type f | sed "s|^$DEST/||" | sort > "$DEST/.CONTENTS"

# Final messages
echo "Bundle created at: $DEST"
ls -la "$DEST/lib" || true
ls -la "$DEST/lib/pkgconfig" || true
ls -la "$DEST/include" | head -20 || true

echo "Remember to commit $DEST into the repo if you want it published with the project."
