// Re-export the constructor from the main library
use slop::BondCurveWrapper;

#[no_mangle]
pub fn new() -> BondCurveWrapper {
    slop::new_bond_curve()
}

fn main() {}
