//! Simple standalone test runner 
//! This file runs the tests in simple_test.rs
//! 
//! Compile and run with:
//! rustc test_runner.rs && ./test_runner

mod simple_test {
    include!("src/simple_test.rs");
}

fn main() {
    // Run the tests from simple_test.rs
    simple_test::main();
}
