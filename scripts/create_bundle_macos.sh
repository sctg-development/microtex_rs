#!/usr/bin/env bash
set -euo pipefail

# Creates a dependency bundle suitable for committing under
# dependencies_bundle/macos/intel. Overwrites DEST if it exists.

DEST=${1:-dependencies_bundle/macos/intel}

echo "Preparing bundle at: $DEST"
rm -rf "$DEST"
mkdir -p "$DEST/lib/pkgconfig" "$DEST/include" "$DEST/share/licenses" || true
# Absolute path to the bundle directory — used to rewrite .pc prefixes
BUNDLE_ABS=$(cd "$DEST" && pwd)

# Packages we want to capture from Homebrew pkg-config
# Include transitive dependencies required by Pango (glib/gobject/gio/fribidi, etc.)
PACKAGES=(cairo pango fontconfig freetype2 harfbuzz harfbuzz-gobject pixman-1 libpcre2-8 libpng lzo2 glib-2.0 gobject-2.0 gio-2.0 fribidi libffi zlib bzip2 expat graphite2)

# Helper: copy files preserving structure
copy_libs() {
  local libdir="$1"
  if [ -d "$libdir" ]; then
    echo "Copying libs from $libdir"
    # Skip copying system-protected libs (SIP) from /usr/lib or /System
    if [[ "$libdir" == /usr/lib* ]] || [[ "$libdir" == /System* ]]; then
      echo "Skipping copying system libs from $libdir (protected)"
    else
      shopt -s nullglob
      for f in "$libdir"/*.{dylib,a,so}; do
        dest="$DEST/lib/$(basename "$f")"
        # remove existing dest (possibly read-only) then copy
        rm -f "$dest" 2>/dev/null || true
        # Dereference symlinks so the bundle contains actual library files (not external symlinks)
        cp -L "$f" "$dest" 2>/dev/null || true
        chmod u+w "$dest" 2>/dev/null || true
      done
      shopt -u nullglob
    fi

    if [ -d "$libdir/pkgconfig" ]; then
      echo "Copying pkgconfig files from $libdir/pkgconfig"
      cp -f "$libdir/pkgconfig"/*.pc "$DEST/lib/pkgconfig/" 2>/dev/null || true
      chmod u+w "$DEST/lib/pkgconfig/"*.pc 2>/dev/null || true
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

  # If pkgconfig directory didn't contain .pc for this package, try common Homebrew/Cellar paths
  if [ ! -f "$DEST/lib/pkgconfig/${pkg}.pc" ] && [ ! -f "$DEST/lib/pkgconfig/${pkg}-1.0.pc" ] && [ ! -f "$DEST/lib/pkgconfig/${pkg}*.pc" ]; then
    echo "Looking for pkgconfig files for $pkg in common Homebrew locations"
    # common locations
    for d in "/opt/homebrew/lib/pkgconfig" "/usr/local/lib/pkgconfig" "/opt/homebrew/Cellar" "/usr/local/Cellar"; do
      if [ -d "$d" ]; then
        # Search for matching pc files (portable loop; avoid mapfile)
        for pc in $(find "$d" -maxdepth 3 -type f -name "${pkg}*.pc" 2>/dev/null || true); do
          echo "Copying pkgconfig file $pc"
          cp -f "$pc" "$DEST/lib/pkgconfig/" 2>/dev/null || true
          chmod u+w "$DEST/lib/pkgconfig/$(basename "$pc")" 2>/dev/null || true
        done
      fi
    done
  fi

  # Try to copy license files if available nearby
  if [ -n "$libdir" ]; then
    for lic in "$libdir"/../share/licenses/$pkg/*; do
      [ -e "$lic" ] && cp -a "$lic" "$DEST/share/licenses/" 2>/dev/null || true
    done
  fi
done

# Rewrite .pc files to use relocatable paths relative to the .pc file (no absolute local paths)
# Use ${pcfiledir} which pkg-config defines to the directory containing the .pc file.
if [ -d "$DEST/lib/pkgconfig" ]; then
  echo 'Rewriting .pc files to use relocatable prefix based on ${pcfiledir}'
  for pc in "$DEST"/lib/pkgconfig/*.pc; do
    [ -f "$pc" ] || continue
    tmp=$(mktemp)
    # Use sed with single-quoted script so the shell does NOT expand ${pcfiledir} nor ${prefix}.
    sed -E '
      s|^prefix=.*|prefix=${pcfiledir}/../..|
      s|^includedir=.*|includedir=${prefix}/include|
      s|^libdir=.*|libdir=${prefix}/lib|
    ' "$pc" > "$tmp" && mv -f "$tmp" "$pc" && chmod 644 "$pc" 2>/dev/null || true
  done
fi

# Post-check: ensure important pkgconfig files exist
REQUIRED_PCS=(pango pangocairo cairo fontconfig)
MISSING_PCS=()
for req in "${REQUIRED_PCS[@]}"; do
  found=false
  # Check for common basename variants
  for pat in "$DEST/lib/pkgconfig/${req}.pc" "$DEST/lib/pkgconfig/${req}-1.0.pc"; do
    if [ -f "$pat" ]; then
      found=true
      break
    fi
  done
  if ! $found; then
    # also accept files that start with req (e.g. pangocairo-1.0.pc)
    if compgen -G "$DEST/lib/pkgconfig/${req}*.pc" > /dev/null; then
      found=true
    fi
  fi
  if ! $found; then
    MISSING_PCS+=($req)
  fi
done

if [ ${#MISSING_PCS[@]} -ne 0 ]; then
  echo "ERROR: The bundle is missing pkgconfig files for: ${MISSING_PCS[*]}" >&2
  echo "Please ensure these packages are installed locally and re-run the script, or run the capture workflow on macOS to produce a complete bundle." >&2
  echo "Contents of $DEST/lib/pkgconfig:" >&2
  ls -la "$DEST/lib/pkgconfig" >&2 || true
  exit 1
fi

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
