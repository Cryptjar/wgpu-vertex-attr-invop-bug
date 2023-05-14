#!/bin/bash

# Build the web-app (aka `wasm32-unknown-unknown`)
# start with `--release` for release mode
#
# Use the environment variable `FEATURES` to enable any features (use comma to
# separate multiple)

set -e

target_name="debug"
build_flags=""
bindgen_flags="--debug --keep-debug" # Default to debugging

while [ $# -ge 1 ]
do
	if [ "$1" == "--release" ]
	then
		shift
		target_name="release"
		build_flags="--release" #  -Z build-std=std,panic_abort
		bindgen_flags="--remove-name-section --remove-producers-section"
	else
		echo "Error: invalid argument"
		exit 1
	fi
done


APP_NAME="wgpu-vertex-attr-invop-bug"

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TARGET_DIR="$SCRIPT_DIR/target"
OUT_DIR="$TARGET_DIR/web-pkg"
ARCH="wasm32-unknown-unknown"
FEAT=""

# Currently, we need the `css-size` winit feature, which in turn requires the
# `web-sys/unstable-apis` feature, which requires us to specfiy the following
# environment variable whenever we compile for WASM:
export RUSTFLAGS=--cfg=web_sys_unstable_apis

if [ -n "$FEATURES" ]
then
	FEAT="--features $FEATURES"
fi

if [[ $target_name = "release" ]]
then
	echo "Cleaning..."
	# First clean that target to ensure that we get a fresh build
	cargo clean --package "$APP_NAME" --target $ARCH $build_flags

	rm -rf "$OUT_DIR"
fi

echo "Create output dir '$OUT_DIR'"
mkdir -p "$OUT_DIR"

# Build wasm binary and binding JS
echo "Build WASM in $target_name mode"
cargo build --package "$APP_NAME" --target $ARCH $build_flags $FEAT
echo "Execute wasm-bindgen"
wasm-bindgen "$TARGET_DIR/$ARCH/$target_name/$APP_NAME.wasm" --out-dir "$OUT_DIR" --target web --no-typescript $bindgen_flags

# Copy web assets
echo "Copy assets"
cp -r "$SCRIPT_DIR/static/"* "$OUT_DIR"

echo "Done"

# Start server:
# simple-http-server --index --nocache target/web-pkg/
