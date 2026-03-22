#!/bin/bash
set -e

APP_NAME="Kode"
BUNDLE_DIR="target/release/${APP_NAME}.app"
BINARY="target/release/kode"

echo "Building release binary..."
cargo build --release

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

echo "Creating app bundle..."

# Clean previous bundle
rm -rf "${BUNDLE_DIR}"

# Create bundle structure
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

# Copy binary — launch with --gpu flag
cat > "${BUNDLE_DIR}/Contents/MacOS/kode-launcher" << 'LAUNCHER'
#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
exec "$DIR/kode" --gpu "$@"
LAUNCHER
chmod +x "${BUNDLE_DIR}/Contents/MacOS/kode-launcher"

# Copy the actual binary
cp "${BINARY}" "${BUNDLE_DIR}/Contents/MacOS/kode"

# Create Info.plist
cat > "${BUNDLE_DIR}/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Kode</string>
    <key>CFBundleDisplayName</key>
    <string>Kode</string>
    <key>CFBundleIdentifier</key>
    <string>com.soulduse.kode</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleExecutable</key>
    <string>kode-launcher</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
</dict>
</plist>
PLIST

# Copy icon if exists
if [ -f "assets/icon.icns" ]; then
    cp "assets/icon.icns" "${BUNDLE_DIR}/Contents/Resources/AppIcon.icns"
fi

echo "Bundle created: ${BUNDLE_DIR}"
echo "Run: open ${BUNDLE_DIR}"
