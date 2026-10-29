#!/usr/bin/env bash
# Regenerates docs/screenshots/*.png under a virtual X display.
# Needs: Xvfb, ImageMagick (import, convert) and Mesa (software OpenGL is fine).
set -euo pipefail

cd "$(dirname "$0")/.."
cargo build -p viewer
out=docs/screenshots
mkdir -p "$out"

display=:${SCREENSHOT_DISPLAY:-99}
Xvfb "$display" -screen 0 1200x800x24 >/dev/null 2>&1 &
xvfb=$!
trap 'kill $xvfb 2>/dev/null || true' EXIT
sleep 1

shot() {
  local name=$1
  shift
  DISPLAY=$display ./target/debug/viewer --spin 0,0,0,0,0,0 "$@" >/dev/null 2>&1 &
  local pid=$!
  sleep 6
  DISPLAY=$display import -window root "$out/$name.png"
  convert "$out/$name.png" -crop 1200x800+0+0 +repage "$out/$name.png"
  kill "$pid"
  wait "$pid" 2>/dev/null || true
}

shot 1-default
shot 2-xw-rotation --angles 0,0,0.9,0,0,0
shot 3-faces-off --angles 0,0,0.9,0,0,0 --no-faces
shot 4-orthographic --angles 0,0,0.9,0,0,0 --ortho
echo "wrote $out/*.png"
