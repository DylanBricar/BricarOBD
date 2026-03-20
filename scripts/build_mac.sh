#!/bin/bash
# Build BricarOBD.app and .dmg for macOS
set -e

echo "=== BricarOBD macOS Build ==="

# Check dependencies
python3 -c "import PyInstaller" 2>/dev/null || {
    echo "Installing PyInstaller..."
    pip3 install pyinstaller
}

# Clean previous builds
rm -rf build/ dist/

# Build .app bundle
echo "Building .app..."
pyinstaller build.spec --noconfirm

echo "✓ App built: dist/BricarOBD.app"

# Create .dmg
echo "Creating .dmg..."
DMG_NAME="BricarOBD-v1.0.0-macOS.dmg"

# Create temp DMG folder
mkdir -p dist/dmg
cp -R dist/BricarOBD.app dist/dmg/
ln -s /Applications dist/dmg/Applications

# Create DMG
hdiutil create -volname "BricarOBD" \
    -srcfolder dist/dmg \
    -ov -format UDZO \
    "dist/$DMG_NAME"

rm -rf dist/dmg

echo ""
echo "=== Build Complete ==="
echo "  .app: dist/BricarOBD.app"
echo "  .dmg: dist/$DMG_NAME"
echo ""
echo "To install: Open the .dmg and drag BricarOBD to Applications"
