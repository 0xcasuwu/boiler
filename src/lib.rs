//! SLOP - Smart Contract Launchpad for Orbital Bonds
//!
//! This library implements a launchpad for creating and managing bond collections
//! where bonds are represented as orbital tokens. The orbital tokens themselves ARE the bonds,
//! functioning as both the financial instrument and the authentication mechanism.
//!
//! ## Key Components
//!
//! * **LaunchpadFactory**: Creates and manages bond collections
//! * **OrbitalBondCollection**: Manages a collection of orbital bonds
//! * **Bond**: Core model representing bond financial attributes
//!
//! ## Usage Overview
//!
//! 1. Create a LaunchpadFactory
//! 2. Create one or more bond collections with desired parameters
//! 3. Mint orbital tokens (which ARE bonds) for users who deposit funds
//! 4. When bonds mature, owners redeem them by presenting their orbital tokens
//!
//! ## Authentication
//!
//! This system leverages orbitals as both bonds and authentication tokens.
//! When a user mints a bond, they receive an orbital token that represents
//! and IS the bond itself. To redeem the bond at maturity, they simply present
//! the orbital token, which proves ownership without requiring user addresses.

// Re-export key modules
pub mod contracts;
pub mod models;
pub mod utils;
#[cfg(test)]
mod tests;

// Re-export important types
pub use contracts::{LaunchpadFactory, OrbitalBondCollection};
pub use models::{Bond, BondStatus};
