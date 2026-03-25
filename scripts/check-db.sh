#!/bin/bash
# Check if the database exists, if not download from latest GitHub release
DB_PATH="src-tauri/data/bricarobd.db"

if [ -f "$DB_PATH" ]; then
    echo "Database found: $DB_PATH ($(du -h "$DB_PATH" | cut -f1))"
    exit 0
fi

echo "Database not found at $DB_PATH"
echo "Please download bricarobd.db from the GitHub releases page"
echo "and place it in src-tauri/data/"
echo ""
echo "Or run: gh release download --pattern 'bricarobd.db' --dir src-tauri/data/"
exit 1
