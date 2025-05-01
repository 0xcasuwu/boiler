// Re-export the constructor from the main library
use slop::LaunchpadFactoryWrapper;

#[no_mangle]
pub fn new() -> LaunchpadFactoryWrapper {
    slop::new_launchpad_factory()
}

fn main() {}
