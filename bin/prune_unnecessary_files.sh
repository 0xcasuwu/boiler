#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}===== Pruning Unnecessary Files =====${NC}"

# List of files to prune
FILES_TO_PRUNE=(
    "src/lib_test.rs"
    "src/lib_tests.rs"
    "src/simple_test.rs"
)

# Create archive directory if it doesn't exist
ARCHIVE_DIR="archive/deprecated_files"
mkdir -p "$ARCHIVE_DIR"

# Move files to archive
for file in "${FILES_TO_PRUNE[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${YELLOW}Moving $file to $ARCHIVE_DIR/${NC}"
        mv "$file" "$ARCHIVE_DIR/"
        echo -e "${GREEN}✓ $file moved${NC}"
    else
        echo -e "${RED}✗ $file not found${NC}"
    fi
done

echo -e "\n${BLUE}${BOLD}===== Summary =====${NC}"
echo -e "${GREEN}Pruned unnecessary files:${NC}"
for file in "${FILES_TO_PRUNE[@]}"; do
    echo -e "- $file"
done

echo -e "\n${YELLOW}Files were moved to $ARCHIVE_DIR/ for reference${NC}"
echo -e "${BLUE}${BOLD}===== Done =====${NC}"
