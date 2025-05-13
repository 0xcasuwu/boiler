use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::context::Context;
use alkanes_support::response::CallResponse;
use anyhow::Result;

// A simple struct to test the dispatch mechanism
struct TestStruct {}

impl Default for TestStruct {
    fn default() -> Self {
        Self {}
    }
}

// Implement AlkaneResponder for TestStruct
impl AlkaneResponder for TestStruct {
    fn height(&self) -> u64 {
        0
    }
    
    fn context(&self) -> Result<Context, anyhow::Error> {
        Ok(Context::default())
    }
}

// Implement a simple hello method
impl TestStruct {
    pub fn hello(&self, name: String) -> Result<CallResponse> {
        let mut response = CallResponse::default();
        response.data = format!("Hello, {}!", name).as_bytes().to_vec();
        Ok(response)
    }
    
    // Simple dispatch method
    pub fn dispatch_message(&self, opcode: u32, args: &[u8]) -> Result<CallResponse> {
        match opcode {
            0 => {
                let name = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.hello(name)
            },
            _ => Err(anyhow::anyhow!("Unknown opcode: {}", opcode))
        }
    }
}

fn main() {
    println!("Testing simple dispatch mechanism");
    
    // Create a test instance
    let test = TestStruct::default();
    
    // Test the dispatch
    let args = b"World";
    match test.dispatch_message(0, args) {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => println!("Error: {}", e),
    }
}
