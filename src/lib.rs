extern crate alkanes_runtime;
extern crate serde;
extern crate serde_json;
extern crate anyhow;

pub mod contracts;
pub mod models;
pub mod utils;
pub mod tests;
pub mod interfaces;
pub mod helpers;

// Re-export key types that are commonly used
pub use alkanes_runtime::message::MessageDispatch;
pub use alkanes_support::context::Context;
pub use alkanes_support::parcel::AlkaneTransferParcel;
pub use alkanes_runtime::runtime::AlkaneResponder;
pub use alkanes_support::response::CallResponse;

pub use anyhow::{
    Result,
    anyhow,
};

// Re-export helpers module
pub use helpers::*;

// Re-export interfaces for binary wrappers
pub use interfaces::bond_curve::BondCurveWrapper;
pub use interfaces::launchpad_factory::LaunchpadFactoryWrapper;
pub use interfaces::orbital_bond::OrbitalBondWrapper;

// Re-export constructor functions for WebAssembly exports
pub fn new_bond_curve() -> BondCurveWrapper {
    BondCurveWrapper::new()
}

pub fn new_launchpad_factory() -> LaunchpadFactoryWrapper {
    LaunchpadFactoryWrapper::new()
}

pub fn new_orbital_bond_collection() -> OrbitalBondWrapper {
    OrbitalBondWrapper::new()
}
