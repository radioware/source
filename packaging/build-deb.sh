#!/usr/bin/env bash
#
# Build a .deb inside a fresh Debian/Ubuntu container. Intended to be run by CI
# (see .github/workflows/release.yml) via:
#
#   docker run --platform <plat> -v "$PWD:/src" -w /src <image> \
#       bash packaging/build-deb.sh <label>
#
# <label> is a short distro tag (e.g. debian12) added to the output filename so
# artifacts from different distros/arches don't collide. Packages are written to
# ./dist.
#
# The Rust toolchain, cargo registry and build output are kept under
# ./.ci-cache and ./target (both inside the bind-mounted workspace) so CI can
# cache them across runs and avoid recompiling cargo-deb every time.
#
set -euo pipefail

LABEL="${1:?usage: build-deb.sh <label>}"

export DEBIAN_FRONTEND=noninteractive
export CARGO_HOME="$PWD/.ci-cache/cargo"
export RUSTUP_HOME="$PWD/.ci-cache/rustup"
export PATH="$CARGO_HOME/bin:$PATH"

echo "::group::Install build dependencies"
apt-get update
apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libopus-dev
echo "::endgroup::"

echo "::group::Install Rust toolchain"
# Minimal stable toolchain; edition 2024 needs Rust >= 1.85. Skipped when a
# cached toolchain is already present.
if ! command -v rustc >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable --no-modify-path
fi
rustc --version
echo "::endgroup::"

echo "::group::Install cargo-deb"
# Skipped when restored from cache.
if ! command -v cargo-deb >/dev/null 2>&1; then
    cargo install cargo-deb --locked
fi
echo "::endgroup::"

echo "::group::Build package"
# Drop any stale packages from a previous cached run so only the fresh build is
# collected below.
rm -rf target/debian
cargo deb
echo "::endgroup::"

echo "::group::Collect artifacts"
mkdir -p dist
for f in target/debian/*.deb; do
    base="$(basename "$f" .deb)"
    # base already contains name_version_arch; append the distro label.
    cp -v "$f" "dist/${base}_${LABEL}.deb"
done
echo "::endgroup::"

# The container runs as root; make cached files world-readable so the non-root
# host runner can archive them for the cache and artifacts.
chmod -R a+rX .ci-cache target dist 2>/dev/null || true
