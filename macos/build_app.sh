#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

FEATURES="${NF_CARGO_FEATURES:-movies}"
FEAT_ARGS=()
if [[ -n "$FEATURES" ]]; then
  FEAT_ARGS=(--features "$FEATURES")
fi

echo "[macos] Building NightfirePort, nf-view, and nf-macos-harness (release)..."
cd "$ROOT"
cargo build --release -p nf-view --bin NightfirePort --bin nf-view "${FEAT_ARGS[@]}"
cargo build --release -p nf-macos --bin nf-macos-harness

APP="$ROOT/macos/NightfirePort.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "$ROOT/target/release/NightfirePort" "$APP/Contents/MacOS/NightfirePort"
cp "$ROOT/target/release/nf-view" "$APP/Contents/MacOS/nf-view"
chmod +x "$APP/Contents/MacOS/NightfirePort" "$APP/Contents/MacOS/nf-view"

cat > "$APP/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>NightfirePort</string>
  <key>CFBundleIdentifier</key>
  <string>com.nightfire.macos</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>NightfirePort</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key>
      <string>PS2 Disc Image</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>iso</string>
        <string>bin</string>
      </array>
    </dict>
  </array>
</dict>
</plist>
PLIST

echo "[macos] Built $APP"
echo "[macos] CLI harness: $ROOT/target/release/nf-macos-harness"
