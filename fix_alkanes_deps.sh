#!/bin/bash

# Set color codes for better readability
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}============================================================${NC}"
echo -e "${BLUE}     Fixing Alkanes Dependencies for SLOP Project ${NC}"
echo -e "${BLUE}============================================================${NC}"

ALKANES_PATH="../alkanes-rs"
TARGET_PATH="../alkanes-rs/target/package"

# Check if alkanes-rs directory exists
if [ ! -d "$ALKANES_PATH" ]; then
    echo -e "${RED}Error: alkanes-rs directory not found at $ALKANES_PATH${NC}"
    echo -e "${YELLOW}Make sure the alkanes-rs repository is cloned to the correct location${NC}"
    exit 1
fi

# Create target directory if it doesn't exist
mkdir -p "$TARGET_PATH"

echo -e "${YELLOW}Building alkanes and related packages...${NC}"
(cd "$ALKANES_PATH" && cargo package --allow-dirty) || {
    echo -e "${RED}Error: Failed to package alkanes crates${NC}"
    exit 1
}

# Function to create symbolic links with correct names
create_symlinks() {
    local source_crate="$1"
    local target_crate="$2"
    
    echo -e "${YELLOW}Creating symlink: $target_crate -> $source_crate${NC}"
    
    if [ -d "$TARGET_PATH/$source_crate" ]; then
        ln -sf "$TARGET_PATH/$source_crate" "$TARGET_PATH/$target_crate"
        echo -e "${GREEN}Successfully created symlink for $target_crate${NC}"
    else
        echo -e "${RED}Error: Source crate $source_crate not found${NC}"
    fi
}

# Create needed symbolic links (with underscore names pointing to hyphenated versions)
create_symlinks "alkanes-runtime" "alkanes_runtime"
create_symlinks "alkanes-support" "alkanes_support"
create_symlinks "protorune-support" "protorune_support"
create_symlinks "protorune-support" "metashrew_support"
create_symlinks "protorune-support" "metashrew-support"

echo -e "${GREEN}Dependency linking completed!${NC}"

echo -e "${YELLOW}Updating cargo configuration to use local dependencies...${NC}"
# Create .cargo/config.toml if it doesn't exist
mkdir -p .cargo
cat > .cargo/config.toml << EOF
[source.crates-io]
replace-with = "local-registry"

[source.local-registry]
directory = "$TARGET_PATH"
EOF

echo -e "${GREEN}Cargo configuration updated!${NC}"
echo -e "${BLUE}============================================================${NC}"
echo -e "${GREEN}Dependencies fixed and configured successfully!${NC}"
echo -e "${BLUE}============================================================${NC}"

echo -e "${YELLOW}You can now build the project with:${NC}"
echo -e "${BLUE}cargo build --features blockchain${NC}"
