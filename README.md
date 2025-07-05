
# Swamp - ICP Blockchain Data Visualization

A blockchain data visualization tool for the Internet Computer Protocol (ICP) ecosystem.

## Setup Guide

### Prerequisites
- Rust (latest stable version)
- Node.js (v16 or higher)
- Cargo
- npm

### Setting up from JSONL Ledger Files

If you have ICP ledger data in JSONL format, follow these steps to import it into a SQLite database and generate visualization data.

#### 1. Import JSONL files into SQLite Database

##### Mac/Linux:
```bash
cd backend

# Import from specific directory
cargo run import_db ~/Downloads/ledger_data ./ledger.db

# Or use default paths (current directory for JSONL files)
cargo run import_db
```

##### Windows:
```bash
cd backend

# Import from specific directory
cargo run import_db C:\Users\YourName\Downloads\ledger_data .\ledger.db

# Or use default paths (current directory for JSONL files)
cargo run import_db
```

**What happens during import:**
- Parses all JSONL files in the specified directory
- Creates SQLite database with indexed transactions table
- Processes transactions (Transfer, Mint, Burn operations)
- Shows progress every 100,000 transactions
- Creates indexes for fast queries

**Expected output:**
```
Importing JSONL files from: /path/to/ledger_data
Creating database at: ./ledger.db
Processing file: transactions_1.jsonl
  Imported 100000 transactions...
  Imported 200000 transactions...
...
Import completed! Total transactions: 25,400,000
```

#### 2. Generate Daily Balance Data

After creating the database, generate the daily balance visualization data:

##### Mac/Linux:
```bash
# Generate daily balances (outputs to ../graph/public/daily_balances.json)
cargo run daily_balances ./ledger.db

# Or use default database path
cargo run daily_balances
```

##### Windows:
```bash
# Generate daily balances (outputs to ..\graph\public\daily_balances.json)
cargo run daily_balances .\ledger.db

# Or use default database path
cargo run daily_balances
```

#### 3. Verify Database (Optional)

Test the database by querying a specific account:

##### Mac/Linux:
```bash
# Query specific account
cargo run query_db 014d583dffef4783812768f349f368f9c18c6c47b86911652aedb6b5cc608b1d ./ledger.db
```

##### Windows:
```bash
# Query specific account
cargo run query_db 014d583dffef4783812768f349f368f9c18c6c47b86911652aedb6b5cc608b1d .\ledger.db
```

### Backend Commands

```bash
cd backend

# Generate graph data from API
cargo run graph_data

# Analyze all suspect accounts for patterns
cargo run analyze_patterns

# Analyze a specific account
cargo run analyze_account <account_hex>

# Trace money flow network
cargo run trace_network

# Other available commands - see CLAUDE.md for full list
```

### Frontend Setup

```bash
cd graph

# Install dependencies
npm install

# Run development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to see the visualization.

### Deployment (Internet Computer)

```bash
cd graph

# Build for IC
dfx build --ic

# Deploy/upgrade canister
dfx canister install graph --ic --mode=upgrade
```

## Features

- **Daily Balance Charts**: Track balance changes over time for pattern addresses
- **Cumulative Total Graph**: See the total ICP held by all tracked addresses
- **Date Selection**: Pick specific dates to analyze balances
- **Interactive Visualizations**: D3.js-powered charts with zoom and pan capabilities

## Troubleshooting

### Import Issues
- Ensure JSONL files are valid JSON format with one transaction per line
- Check that you have enough disk space for the SQLite database (typically 2-3GB for full ledger)
- If import fails, delete the partial `ledger.db` file and try again

### Performance
- The import process may take 10-30 minutes depending on data size
- Daily balance generation typically takes 1-2 minutes
- For large datasets, ensure you have at least 8GB RAM available

