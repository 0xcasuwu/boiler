# Subfrost RPC Extension Implementation Guide

This guide provides detailed instructions for implementing the necessary RPC extensions to enable SLOP contract deployment in the Subfrost environment. It serves as a practical companion to the theoretical specifications in `subrail-extension.md`.

## Prerequisites

- Access to the Subfrost repository
- Docker installed and configured
- Go development environment (for testing modifications)
- Basic understanding of Docker and Go programming

## Implementation Steps

### 1. Fork and Clone the Subfrost Repository

```bash
# Create a fork of the repository through GitHub UI first, then:
git clone https://github.com/[your-username]/subfrost.git
cd subfrost
git checkout -b feature/extend-rpc-methods
```

### 2. Modify the Dockerfile.subrail File

Open the `Dockerfile.subrail` file and make the following changes to extend the RPC server:

#### 2.1 Add Global State Variables

Find the existing global state variables section and add these new variables:

```go
// Global state
var (
    currentHeight int = 840000
    contextAddress string = "0x0000000000000000000000000000000000000000"
    psbtStore = make(map[int][]string)
    
    // New state for contract deployment and interaction
    deployedContracts = make(map[string]string)    // txid -> contract binary
    contractCalldata = make(map[string]string)     // txid -> calldata
    contractResults = make(map[string]interface{}) // txid -> call results
    nextTxid = 1000  // Starting point for transaction IDs
)
```

#### 2.2 Add Helper Functions

Add these helper functions after the existing ones:

```go
// Generate a mock transaction ID for contract deployments
func generateTxid() string {
    txid := fmt.Sprintf("%064x", nextTxid)
    nextTxid++
    return txid
}

// Helper to convert hex to bytes
func hexToBytes(hex string) ([]byte, error) {
    if len(hex)%2 != 0 {
        return nil, fmt.Errorf("hex string must have even length")
    }
    bytes := make([]byte, len(hex)/2)
    for i := 0; i < len(hex); i += 2 {
        b, err := strconv.ParseUint(hex[i:i+2], 16, 8)
        if err != nil {
            return nil, err
        }
        bytes[i/2] = byte(b)
    }
    return bytes, nil
}
```

#### 2.3 Extend the RPC Method Handler

Find the switch statement in the `handleRPC` function and extend it with these new cases:

```go
switch request.Method {
case "subrail_height":
    response.Result = currentHeight

case "metashrew_height":  // Alias for subrail_height for compatibility
    response.Result = currentHeight

case "btc_getblockcount":  // Bitcoin node block count (same as our height)
    response.Result = currentHeight

case "alkane_newContract":
    // Parse params: [contractHex, calldata, feeRate]
    var params []interface{}
    err = json.Unmarshal(request.Params, &params)
    if err != nil || len(params) < 2 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params"}
        break
    }
    
    // Extract contract hex and calldata
    contractHex, ok1 := params[0].(string)
    calldata, ok2 := params[1].(string)
    if !ok1 || !ok2 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params types"}
        break
    }
    
    // Generate transaction ID and store contract
    txid := generateTxid()
    deployedContracts[txid] = contractHex
    contractCalldata[txid] = calldata
    
    // Return transaction ID as result
    response.Result = txid

case "alkane_callContract":
    // Parse params: [txid, opcode, args]
    var params []interface{}
    err = json.Unmarshal(request.Params, &params)
    if err != nil || len(params) < 3 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params"}
        break
    }
    
    // Extract txid and opcode
    txidStr, ok1 := params[0].(string)
    opcode, ok2 := params[1].(float64)
    args, ok3 := params[2].(string)
    if !ok1 || !ok2 || !ok3 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params types"}
        break
    }
    
    // Check if contract exists
    _, exists := deployedContracts[txidStr]
    if !exists {
        response.Error = &JsonRpcError{Code: -32000, Message: "Contract not found"}
        break
    }
    
    // Mock successful call and return a new txid
    callTxid := generateTxid()
    contractResults[callTxid] = map[string]interface{}{
        "success": true,
        "opcode": opcode,
        "args": args,
        "timestamp": time.Now().Unix(),
    }
    
    response.Result = callTxid

case "alkane_getTransactionReceipt":
    // Parse params: [txid]
    var params []string
    err = json.Unmarshal(request.Params, &params)
    if err != nil || len(params) < 1 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params"}
        break
    }
    
    txid := params[0]
    
    // Check if contract or result exists
    if _, exists := deployedContracts[txid]; exists {
        response.Result = map[string]interface{}{
            "txid": txid,
            "type": "contract_deployment",
            "status": "confirmed",
            "block_height": currentHeight,
            "timestamp": time.Now().Unix(),
        }
        break
    }
    
    if result, exists := contractResults[txid]; exists {
        response.Result = map[string]interface{}{
            "txid": txid,
            "type": "contract_call",
            "status": "confirmed",
            "block_height": currentHeight,
            "result": result,
            "timestamp": time.Now().Unix(),
        }
        break
    }
    
    // Not found
    response.Result = nil

case "subrail_getBundle":
    // Existing implementation...

case "subrail_getBundleState":
    // Existing implementation...

case "subrail_setContext":
    // Existing implementation...

default:
    response.Error = &JsonRpcError{Code: -32601, Message: "Method not found"}
}
```

#### 2.4 Import Additional Required Packages

Add any additional imports at the top of the file if needed:

```go
import (
    "encoding/json"
    "flag"
    "fmt"
    "log"
    "net/http"
    "io/ioutil"
    "math/rand"
    "strconv"  // New import
    "time"
)
```

### 3. Build the Modified Docker Image

```bash
# Navigate to the subfrost directory
cd /path/to/subfrost

# Build the modified Docker image
docker build -t subfrost-subrail-extended -f Dockerfile.subrail .
```

### 4. Update Docker Compose Configuration

Modify the `docker-compose.yml` file to use your extended subrail image:

```yaml
# Find the subrail service and update the build section:
subrail:
  image: subfrost-subrail-extended
  # or if using build context:
  build:
    context: .
    dockerfile: Dockerfile.subrail
  container_name: subfrost-subrail
  # Rest of the configuration stays the same
```

### 5. Deploy and Test the Environment

```bash
# Stop any running instances
docker-compose down

# Start the environment with the new configuration
docker-compose up -d
```

### 6. Test the RPC Methods

Create a test script `test-rpc-methods.sh` to verify the implementation:

```bash
#!/bin/bash

# Terminal colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

RPC_URL="http://localhost:18888"

test_method() {
  local method=$1
  local params=$2
  local name=$3
  
  echo "Testing $name..."
  local result=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}")
  
  if echo "$result" | grep -q "error"; then
    echo -e "${RED}Failed:${NC} $method returned an error:"
    echo "$result" | jq
    return 1
  else
    echo -e "${GREEN}Success:${NC} $method returned:"
    echo "$result" | jq
    return 0
  fi
}

echo "Starting RPC method tests..."

# Test metashrew_height
test_method "metashrew_height" "[]" "Block Height (metashrew)"

# Test btc_getblockcount
test_method "btc_getblockcount" "[]" "Block Height (Bitcoin)"

# Test alkane_newContract
CONTRACT_HEX="70736274ff00"  # Minimal valid hex
test_method "alkane_newContract" "[\"$CONTRACT_HEX\",\"3,100001,\\\"1.0.0\\\",1\",5]" "Contract Deployment"

# Extract txid from previous result for next test
TXID=$(echo "$result" | jq -r '.result')

# Test alkane_callContract
test_method "alkane_callContract" "[\"$TXID\",77,\"bcrt1qcr8te4kr609gcawutmrza0j4xv80jy8zeqchgx,1000000\"]" "Contract Interaction"

# Test alkane_getTransactionReceipt
CALL_TXID=$(echo "$result" | jq -r '.result')
test_method "alkane_getTransactionReceipt" "[\"$CALL_TXID\"]" "Transaction Receipt"

echo "RPC method tests complete!"
```

Make the script executable and run it:

```bash
chmod +x test-rpc-methods.sh
./test-rpc-methods.sh
```

### 7. Deploy SLOP Contracts

If all tests pass, you can now deploy the SLOP contracts:

```bash
# Navigate to the boiler repository
cd /Users/erickdelgado/Documents/GitHub/boiler

# Make sure the deployment script is executable
chmod +x deploy-slop-subfrost.sh

# Run the deployment
./deploy-slop-subfrost.sh
```

## Troubleshooting

### RPC Connection Issues

If you encounter RPC connection issues:

1. Check if the Docker containers are running:
   ```bash
   docker ps | grep subfrost
   ```

2. Verify the logs for any errors:
   ```bash
   docker logs subfrost-subrail
   ```

3. Check if the port is accessible:
   ```bash
   curl -X GET http://localhost:18888/health
   ```

### Contract Deployment Failures

If contract deployment fails:

1. Verify the contract binary is valid WebAssembly
2. Check the format of the calldata parameters
3. Ensure the RPC methods are correctly implemented

## Next Steps After Implementation

1. Contribute the changes back to the Subfrost repository as a pull request
2. Document the enhanced RPC capabilities for future users
3. Consider implementing more comprehensive contract state storage

By following this guide, you should be able to extend the Subfrost environment to support SLOP contract deployment and interaction.
