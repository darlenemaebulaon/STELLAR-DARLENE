# GasoLinx PH
DeCentralized Wholesale Crowdsourced Fuel Purchasing for Independent Gas Stations in the Philippines.

## Problem & Solution
Independent provincial gas stations in the Philippines face extreme capital shortfalls when forced to pay bulk distributor delivery orders upfront in flat cash. GasoLinx solves this liquidity barrier by allowing local drivers to secure discounted fuel vouchers upfront, pooling capital via Soroban smart contract storage to transparently aggregate bulk inventory supply.

## Timeline
*   **Day 1-2:** Core Soroban Escrow Strategy Architecture and Unit Testing completion.
*   **Day 3-4:** Frontend interface development & TypeScript SDK bindings integration.
*   **Day 5:** Live Stellar Testnet integration validation and Demo delivery pitch preparation.

## Stellar Features Used
*   Soroban Native Smart Contracts
*   Stellar Asset Standard token framework (USDC/PHPC tracking)
*   Account native cryptographic signatures (`require_auth`)

## Prerequisites
*   Rust toolchain (v1.75+)
*   Stellar CLI (v22+)
*   Target configuration setup: `rustup target add wasm32-unknown-unknown`

## Build & Test Instructions

```bash
# Build the production target Wasm artifact
stellar contract build

# Execute cargo testing suite parameters 
cargo test