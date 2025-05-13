use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::message::MessageDispatch;
use alkanes_support::context::Context;
use alkanes_support::response::CallResponse;
use anyhow::Result;

// A simple struct to test the declare_alkane macro
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

// Define a message enum for TestStruct
#[derive(alkanes_proc_macros::MessageDispatch)]
pub enum TestMessage {
    #[opcode(0)]
    Hello {
        name: String,
    },
    
    #[opcode(1)]
    Add {
        a: u128,
        b: u128,
    },
}

// Implement the handler methods for TestStruct
impl TestStruct {
    pub fn hello(&self, name: String) -> Result<CallResponse, anyhow::Error> {
        let mut response = CallResponse::default();
        response.data = format!("Hello, {}!", name).as_bytes().to_vec();
        Ok(response)
    }
    
    pub fn add(&self, a: u128, b: u128) -> Result<CallResponse, anyhow::Error> {
        let mut response = CallResponse::default();
        response.data = format!("{}", a + b).as_bytes().to_vec();
        Ok(response)
    }
}

// Use the declare_alkane macro to generate the dispatch_message method
alkanes_proc_macros::declare_alkane! {
    impl AlkaneResponder for TestStruct {
        type Message = TestMessage;
    }
}

fn main() {
    println!("Testing declare_alkane macro");
    
    // Create a test instance
    let test = TestStruct::default();
    
    // Test the hello message
    let hello_args = "World".as_bytes();
    match test.dispatch_message(0, hello_args) {
        Ok(response) => {
            let response_str = String::from_utf8(response.data).unwrap_or_default();
            println!("Hello response: {}", response_str);
        },
        Err(e) => println!("Error: {}", e),
    }
    
    // Test the add message
    // Create a buffer with two u128 values (1 and 2)
    let mut add_args = Vec::new();
    add_args.extend_from_slice(&1u128.to_le_bytes());
    add_args.extend_from_slice(&2u128.to_le_bytes());
    
    match test.dispatch_message(1, &add_args) {
        Ok(response) => {
            let response_str = String::from_utf8(response.data).unwrap_or_default();
            println!("Add response: {}", response_str);
        },
        Err(e) => println!("Error: {}", e),
    }
}
