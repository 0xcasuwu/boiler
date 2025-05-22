# Vault Factory

A reward vesting system built on Alkanes that follows the factory-child pattern.

## Overview

This system allows users to deposit assets to earn time-based rewards. The factory creates position tokens that represent a user's deposit, and rewards are calculated based on block height differences.

## Architecture

- **Factory Contract**: Creates and manages position tokens, handles reward distribution
- **Position Tokens**: NFT-like tokens that represent a user's deposit and track individual state
- **Authentication**: Token-based with registry verification for security

## Key Features

- Time-based reward vesting
- Secure cross-contract communication
- Optimized position data access with packed data format
- Factory-controlled critical state updates

## Workflow

1. Factory is initialized with a reward token ID and vesting parameters
2. Users deposit assets and receive a position token
3. Rewards accrue based on time elapsed (block height) and deposit amount
4. Users can claim ongoing rewards or withdraw their entire position

## Security

The implementation uses a dual authentication mechanism:
- Position tokens are verified against the factory's registry
- The factory authenticates itself to position tokens using its token ID
