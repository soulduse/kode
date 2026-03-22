#!/bin/bash
set -e

APP_NAME="Kode"
VERSION="0.1.0"
ARCH=$(uname -m)
DMG_NAME="${APP_NAME}-${VERSION}-macOS-${ARCH}.dmg"
BUNDLE="target/release/${APP_NAME}.app"

# First build the bundle
./scripts/bundle-macos.sh

if [ ! -d "$BUNDLE" ]; then
    echo "Error: Bundle not found at $BUNDLE"
    exit 1
fi

# Check for create-dmg
if ! command -v create-dmg &> /dev/null; then
    echo "create-dmg not found. Install with: brew install create-dmg"
    echo "Falling back to hdiutil..."

    # Simple DMG creation with hdiutil
    DMG_PATH="target/release/${DMG_NAME}"
    rm -f "${DMG_PATH}"

    TEMP_DIR=$(mktemp -d)
    cp -R "${BUNDLE}" "${TEMP_DIR}/"
    ln -s /Applications "${TEMP_DIR}/Applications"

    hdiutil create -volname "${APP_NAME}" \
        -srcfolder "${TEMP_DIR}" \
        -ov -format UDZO \
        "${DMG_PATH}"

    rm -rf "${TEMP_DIR}"
    echo "DMG created: ${DMG_PATH}"
    exit 0
fi

# Create DMG with create-dmg
DMG_PATH="target/release/${DMG_NAME}"
rm -f "${DMG_PATH}"

create-dmg \
    --volname "${APP_NAME}" \
    --window-pos 200 120 \
    --window-size 600 400 \
    --icon-size 100 \
    --icon "${APP_NAME}.app" 175 120 \
    --hide-extension "${APP_NAME}.app" \
    --app-drop-link 425 120 \
    "${DMG_PATH}" \
    "${BUNDLE}"

echo "DMG created: ${DMG_PATH}"
