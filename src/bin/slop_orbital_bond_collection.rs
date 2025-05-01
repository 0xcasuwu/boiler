// Re-export the constructor from the main library
use slop::OrbitalBondWrapper;

#[no_mangle]
pub fn new() -> OrbitalBondWrapper {
    slop::new_orbital_bond_collection()
}

fn main() {}
