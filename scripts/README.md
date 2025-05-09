# Build Scripts Organization

This project uses a carefully designed set of build scripts to handle WebAssembly compilation across different architectures, with special focus on Apple Silicon compatibility.

## Main Build Scripts

### Root Directory

These core build scripts are accessible from the repository root:

1. **build_minimal.sh**
   - **Purpose**: Recommended for Apple Silicon builds
   - **Features**:
     - Creates minimal WebAssembly output file (placeholder if needed)
     - Configures local dependencies
     - Sets proper environment variables for M1/M2/M3 architecture
     - Handles the secp256k1-sys compatibility issues

2. **final_fork_build.sh**
   - **Purpose**: Main build script for standard architectures
   - **Features**:
     - Full build with local dependencies
     - Architecture detection
     - Proper error handling
     - Creates complete WebAssembly binary (~102KB)

3. **build_with_fork.sh**
   - **Purpose**: Advanced control over build process
   - **Features**: 
     - Customizable build parameters
     - Manual dependency management
     - More verbose output
     - For development and debugging use

### Scripts Directory

These helper scripts provide additional functionality:

1. **build_all.sh**
   - **Purpose**: Master orchestration script
   - **Features**:
     - Detects platform and chooses appropriate build script
     - Verifies repository structure
     - Validates build output
     - Provides comprehensive status information

2. **check_mac_m1.sh**
   - **Purpose**: Apple Silicon detection
   - **Features**:
     - Detects M1/M2/M3 architecture
     - Verifies LLVM installation
     - Returns appropriate exit code for automated scripts

## Deployment and Interaction

1. **deploy_to_oylnet.sh**
   - **Purpose**: Deploy to OylNet testnet
   - **Features**:
     - Automated contract deployment
     - Initializes contract parameters
     - Stores contract ID for future reference

2. **interact_with_vault.sh**
   - **Purpose**: Interact with deployed contract
   - **Features**:
     - Tests core functionality
     - Uses numeric parameters for authentication
     - Simplifies contract interaction for testing

## Usage Recommendation

For most users:
1. Run `scripts/build_all.sh` for automatic platform detection and build
2. Use `deploy_to_oylnet.sh` for deployment after successful build
3. Use `interact_with_vault.sh` to test and interact with the contract

For developers with specific needs:
- Use `build_minimal.sh` directly on Apple Silicon machines
- Use `final_fork_build.sh` on standard architectures when more control is needed
- Use `build_with_fork.sh` for advanced debugging and custom configurations
