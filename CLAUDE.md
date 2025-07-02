# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Swamp is a blockchain data visualization tool for the Internet Computer Protocol (ICP) ecosystem. It consists of:
- **Backend**: Rust application that fetches and processes transaction data from the ICP ledger
- **Frontend**: React/TypeScript app that visualizes transaction flows as an interactive D3.js graph

The project tracks and categorizes different account types (CEXes, DeFi protocols, Foundation accounts, Node Providers, SNS projects, and suspicious/spam accounts) to analyze transaction patterns.

## Development Commands

### Backend (Rust)
```bash
cd backend

# Generate graph data
cargo run graph_data

# Analyze all suspect accounts for patterns
cargo run analyze_patterns

# Analyze a specific account by hex address
cargo run analyze_account <account_hex>

# Trace the entire money flow network from seed addresses
cargo run trace_network

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Type check
cargo check
```

### Frontend (React/TypeScript)
```bash
cd graph

# Install dependencies
npm install

# Run development server (http://localhost:3000)
npm start
# or
npm run dev

# Build for production
npm run build

# Run tests
npm test

# The project uses ESLint through react-scripts
```

### Deployment (Internet Computer)
```bash
cd graph

# Build for IC deployment
dfx build --ic

# Deploy/upgrade the canister
dfx canister install graph --ic --mode=upgrade
```

## Architecture

### Backend Structure
- `backend/src/addresses.rs`: Categorized blockchain addresses (CEXes, DeFi, Foundation, etc.)
- `backend/src/transactions.rs`: Fetches transaction data from ICP ledger using ic-agent
- `backend/src/helper.rs`: Address conversion utilities
- `backend/src/main.rs`: Entry point for data processing

### Frontend Structure
- `graph/src/Graph.tsx` & `GraphContainer.tsx`: Core D3.js visualization components
- `graph/src/graphData.ts`: Graph data processing logic
- `graph/src/Ledger.tsx`: Ledger search functionality
- `graph/public/`: Pre-processed transaction data JSON files

### Key Dependencies
- **Backend**: IC SDK (ic-cdk, ic-agent), Tokio for async operations
- **Frontend**: React 19.1, D3.js for visualization, React Router, Bootstrap for UI

## Development Workflow

1. Transaction data is fetched and processed by the Rust backend
2. Processed data is saved as JSON files in `graph/public/`
3. Frontend loads these JSON files and renders interactive graphs
4. The application can be deployed as an Internet Computer canister using dfx

## Code Style

- **Rust**: Uses rustfmt with max width of 120 characters
- **TypeScript**: Strict mode enabled, ES5 target
- **React**: Follows react-app ESLint configuration