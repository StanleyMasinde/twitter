#!/usr/bin/env bash
set -euo pipefail

target="${1:?Usage: build-android.sh TARGET}"
: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to the Android NDK directory}"
toolchain="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"

# API 24 supports all four Termux architectures and Android 7 or later.
case "$target" in
    aarch64-linux-android|x86_64-linux-android|i686-linux-android)
        compiler="$target" ;;
    armv7-linux-androideabi)
        compiler="armv7a-linux-androideabi" ;;
    *) echo "Unsupported Android target: $target" >&2; exit 1 ;;
esac

export PATH="$toolchain:$PATH"
target_key="${target//-/_}"
linker_key=$(printf '%s' "$target_key" | tr '[:lower:]' '[:upper:]')
export "CC_${target_key}=$toolchain/${compiler}24-clang"
export "CXX_${target_key}=$toolchain/${compiler}24-clang++"
export "AR_${target_key}=$toolchain/llvm-ar"
export "CARGO_TARGET_${linker_key}_LINKER=$toolchain/${compiler}24-clang"

# Native libraries are bundled for Android; never discover host Linux libraries.
export PKG_CONFIG_ALLOW_CROSS=0
cargo build --release --locked --target "$target"

# Catch accidental dependencies on runner or Termux-specific shared libraries.
binary="target/$target/release/twitter"
"$toolchain/llvm-readelf" -h "$binary"
needed=$("$toolchain/llvm-readelf" -d "$binary" | sed -n 's/.*Shared library: \[\(.*\)\]/\1/p')
while IFS= read -r library; do
    case "$library" in
        ''|libc.so|libm.so|libdl.so|liblog.so|libz.so) ;;
        *) echo "Unexpected Android shared library: $library" >&2; exit 1 ;;
    esac
done <<< "$needed"
