#!/bin/bash

echo "Checking if this is a Mac M1 system..."

# Check if we're on macOS
if [[ "$(uname)" == "Darwin" ]]; then
    echo "System is macOS"
    
    # Check the CPU type
    CPU_INFO=$(sysctl -n machdep.cpu.brand_string)
    echo "CPU Info: $CPU_INFO"
    
    if [[ "$CPU_INFO" == *"Apple"* ]] && [[ "$CPU_INFO" != *"Intel"* ]]; then
        echo "This appears to be an Apple Silicon (M1/M2) Mac"
        IS_MAC_M1=true
    else
        echo "This appears to be an Intel Mac"
        IS_MAC_M1=false
    fi
else
    echo "Not on macOS"
    IS_MAC_M1=false
fi

# Check if Homebrew LLVM is installed
HOMEBREW_LLVM_PATH="/usr/local/opt/llvm/bin"
if [ -d "$HOMEBREW_LLVM_PATH" ]; then
    echo "Homebrew LLVM is installed at: $HOMEBREW_LLVM_PATH"
    echo "Found clang: $(ls -la $HOMEBREW_LLVM_PATH/clang 2>/dev/null || echo 'Not found')"
    echo "Found llvm-ar: $(ls -la $HOMEBREW_LLVM_PATH/llvm-ar 2>/dev/null || echo 'Not found')"
else
    echo "Homebrew LLVM is NOT installed at: $HOMEBREW_LLVM_PATH"
    
    if [ "$IS_MAC_M1" = true ]; then
        echo ""
        echo "For WebAssembly support on Mac M1, you need to install LLVM via Homebrew:"
        echo "  arch -x86_64 /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\""
        echo "  arch -x86_64 /usr/local/bin/brew install llvm"
        echo "  export PATH=\"/usr/local/opt/llvm/bin:\$PATH\""
    fi
fi
