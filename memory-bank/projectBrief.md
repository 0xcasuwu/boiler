# SLOP Project Brief

## Overview

SLOP (Smart contract Launchpad for Orbital Payments) is a specialized launchpad implementation that enables the creation, management, and redemption of bonds using orbital tokens. It's designed to provide a decentralized way to issue financial instruments that combine bond-like characteristics with the authentication properties of orbitals.

## Core Components

1. **Orbital Bonds**: Tokens that ARE bonds, not just representing them. The orbital token itself functions as both the financial instrument and the authentication mechanism.

2. **LaunchpadFactory**: A factory pattern implementation that creates and manages bond collections, allowing for multiple independent bond issuances.

3. **OrbitalBondCollection**: Individual collections of bonds with specific parameters like interest rates and maturity periods.

4. **Block-Based Maturity**: A mechanism that uses block numbers instead of timestamps for determining bond maturity, providing more reliable and consistent calculations.

## Key Features

1. **Bond Issuance**: Users can deposit diesel tokens to mint orbital bonds with the amount and terms specified by the collection parameters.

2. **Bond Redemption**: Bond holders can redeem their bonds after maturity by presenting the orbital token itself, without requiring user address verification.

3. **Factory Pattern**: Each new bond is a completely new collection issuance from the factory, ensuring isolation and independence.

4. **Context-Based Authentication**: Authentication is performed based on the orbital token itself, making user addresses irrelevant to the authentication process.

5. **Interest Management**: Bonds accrue interest over time until maturity, with configurable rates per collection.

6. **Collection Management**: Ability to create, activate, deactivate, and manage multiple bond collections from a single factory.

## Project Goals

1. **Simplicity**: Create a clean, understandable bond system that doesn't require complex user identification.

2. **Security**: Implement strong security through the orbital token model, where possession of the token constitutes ownership.

3. **Flexibility**: Support multiple independent bond collections with different parameters.

4. **Performance**: Optimize for gas efficiency and computational performance.

5. **Testability**: Implement comprehensive testing to ensure the system works as expected under all conditions.

## Critical Principles

### Orbitals ARE Bonds

**CRITICAL PRINCIPLE**: The orbital token IS the bond, not just a reference to it. This fundamentally changes how we model and interact with bonds, as there is no separate "bond" entity that needs to be linked to an orbital token - the orbital token itself contains all the bond properties and behaviors.

### Address-Free Authentication

**CRITICAL PRINCIPLE**: User addresses should never be considered in the authentication flow. The presence of the orbital token in the transaction context is the only authentication mechanism required. This strengthens the system by removing user identity dependencies.

### Factory Independence

**CRITICAL PRINCIPLE**: Each bond collection created by the factory should be completely independent, with its own state and parameters. This ensures isolation and prevents issues in one collection from affecting others.

## Success Criteria

1. **Functional Completeness**: All specified features are implemented and working correctly.

2. **Test Coverage**: Comprehensive test coverage across all components and scenarios.

3. **Security Verification**: Security audits pass and the system is resistant to common attack vectors.

4. **Performance Optimization**: Efficient gas usage and computational performance.

5. **Documentation**: Complete and accurate documentation of the system's functionality and API.
