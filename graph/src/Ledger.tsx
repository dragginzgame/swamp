import { JSX, useEffect, useState } from "react";
import axios from "axios";
import { AwsClient } from 'aws4fetch'

const apiKey = "";
const apiSecret = "";
// Define TypeScript interfaces for our data structures
interface Tokens {
  e8s: number;
}

interface Operation {
  type: string;
  to?: string;
  from?: string;
  amount?: Tokens;
  fee?: Tokens;
  spender?: string | null;
  allowance?: Tokens;
}

interface Transaction {
  memo: number | string;
  icrc1_memo: string | null;
  operation: Operation;
  created_at_time: number;
}

interface Block {
  transaction: Transaction;
  timestamp: number;
  parent_hash: string | null;
}

interface OpenSearchHit {
  _index: string;
  _id: string;
  _score: number;
  _source: Block;
}

interface OpenSearchResponse {
  took: number;
  timed_out: boolean;
  _shards: {
    total: number;
    successful: number;
    skipped: number;
    failed: number;
  };
  hits: {
    total: {
      value: number;
      relation: string;
    };
    max_score: number;
    hits: OpenSearchHit[];
  };
}

interface SearchState {
  fromAccount: string;
  toAccount: string;
  transactionType: string;
  startDate: string;
  endDate: string;
  tokenAmount: string;
  parentHash: string;
  memo: string;
  usePartialMatch: boolean;
}



const host = "https://search-ic-ledger-ahnchhnlwebccd3svwpa3xx7xe.aos.eu-west-2.on.aws";
const region = "eu-west-2";
const index = 'icp_ledger';
if (!apiKey || !apiSecret) {
  throw new Error("AWS credentials are not set in environment variables");
};
const aws = new AwsClient({ accessKeyId: apiKey, secretAccessKey: apiSecret, region: region, service: 'es' });
const searchUrl = `${host}/${index}/_search?pretty`;

const TRANSACTION_TYPES = [
  "All Types",
  "Transfer",
  "Mint",
  "Burn",
  "Approve"
];

export function Ledger(): JSX.Element {
  // State
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [totalHits, setTotalHits] = useState<number>(0);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const pageSize = 10;
  
  // Search filters
  const [search, setSearch] = useState<SearchState>({
    fromAccount: '',
    toAccount: '',
    transactionType: 'All Types',
    startDate: '',
    endDate: '',
    tokenAmount: '',
    parentHash: '',
    memo: '',
    usePartialMatch: false
  });
  
  // Handle input changes
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>): void => {
    const { name, value, type } = e.target;
    const checked = (e.target as HTMLInputElement).checked;
    
    setSearch({
      ...search,
      [name]: type === 'checkbox' ? checked : value
    });
  };
  
  // Build the search query based on filters
  const buildQuery = (): any => {
    const query: any = {
      bool: {
        must: []
      }
    };
    
    if (search.fromAccount) {
      if (search.usePartialMatch) {
        query.bool.must.push({
          wildcard: { "transaction.operation.from": `*${search.fromAccount}*` }
        });
      } else {
        query.bool.must.push({
          match: { "transaction.operation.from": search.fromAccount }
        });
      }
    }
    
    if (search.toAccount) {
      if (search.usePartialMatch) {
        query.bool.must.push({
          wildcard: { "transaction.operation.to": `*${search.toAccount}*` }
        });
      } else {
        query.bool.must.push({
          match: { "transaction.operation.to": search.toAccount }
        });
      }
    }
    
    if (search.transactionType && search.transactionType !== 'All Types') {
      query.bool.must.push({
        match: { "transaction.operation.type": search.transactionType }
      });
    }
    
    if (search.tokenAmount) {
      query.bool.must.push({
        range: { "transaction.operation.amount.e8s": { gte: parseInt(search.tokenAmount) } }
      });
    }
    
    if (search.startDate || search.endDate) {
      const dateRange: any = { range: { "timestamp": {} } };
      if (search.startDate) {
        dateRange.range.timestamp.gte = new Date(search.startDate).getTime();
      }
      if (search.endDate) {
        dateRange.range.timestamp.lte = new Date(search.endDate).getTime();
      }
      query.bool.must.push(dateRange);
    }
    
    if (search.parentHash) {
      query.bool.must.push({
        match: { "parent_hash": search.parentHash }
      });
    }
    
    if (search.memo) {
      query.bool.must.push({
        match: { "transaction.memo": search.memo }
      });
    }
    
    // If no filters are applied, return match_all
    if (query.bool.must.length === 0) {
      return { match_all: {} };
    }
    
    return query;
  };
  
  // Run the search
  const runSearch = async (page = 1): Promise<void> => {
    setLoading(true);
    setError(null);
    
    try {
      const from = (page - 1) * pageSize;
      const queryBody = {
        query: buildQuery(),
        size: pageSize,
        from: from,
        sort: [
          { "timestamp": { "order": "desc" } }
        ]
      };
      
      const res = await aws.fetch(searchUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(queryBody)
      });
      
      if (!res.ok) {
        throw new Error(`HTTP error ${res.status}: ${res.statusText}`);
      }
      
      const data: OpenSearchResponse = await res.json();
      setBlocks(data.hits.hits.map(hit => hit._source));
      setTotalHits(data.hits.total.value);
      setCurrentPage(page);
    } catch (err) {
      console.error("Search error:", err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };
  
  // Handle search submission
  const handleSubmit = (e: React.FormEvent<HTMLFormElement>): void => {
    e.preventDefault();
    setCurrentPage(1);
    runSearch(1);
  };
  
  // Handle pagination
  const handlePageChange = (page: number): void => {
    setCurrentPage(page);
    runSearch(page);
  };
  
  // Load initial data
  useEffect(() => {
    runSearch();
  }, []);
  
  // Calculate total pages
  const totalPages = Math.ceil(totalHits / pageSize);
  
  // Format e8s tokens to ICP
  const formatICP = (e8s?: number): string => {
    if (e8s === undefined) return 'N/A';
    return (e8s / 100000000).toFixed(8) + ' ICP';
  };
  
  // Format timestamp
  const formatDate = (timestamp?: number): string => {
    if (!timestamp) return 'N/A';
    return new Date(timestamp).toLocaleString();
  };
  
  // Truncate long strings
  const truncate = (str?: string, length = 10): string => {
    if (!str) return 'N/A';
    if (str.length <= length) return str;
    return `${str.substring(0, length)}...`;
  };
  
  // Generate pagination items
  const renderPaginationItems = (): JSX.Element[] => {
    const items: JSX.Element[] = [];
    
    // Previous button
    items.push(
      <li key="prev" className={`page-item ${currentPage === 1 ? 'disabled' : ''}`}>
        <a className="page-link" href="#" onClick={(e) => {
          e.preventDefault();
          if (currentPage > 1) handlePageChange(currentPage - 1);
        }}>
          Previous
        </a>
      </li>
    );
    
    // Page numbers
    let startPage = Math.max(1, currentPage - 2);
    let endPage = Math.min(totalPages, startPage + 4);
    
    if (endPage - startPage < 4 && totalPages > 4) {
      startPage = Math.max(1, endPage - 4);
    }
    
    for (let i = startPage; i <= endPage; i++) {
      items.push(
        <li key={i} className={`page-item ${i === currentPage ? 'active' : ''}`}>
          <a className="page-link" href="#" onClick={(e) => {
            e.preventDefault();
            handlePageChange(i);
          }}>
            {i}
          </a>
        </li>
      );
    }
    
    // Next button
    items.push(
      <li key="next" className={`page-item ${currentPage === totalPages ? 'disabled' : ''}`}>
        <a className="page-link" href="#" onClick={(e) => {
          e.preventDefault();
          if (currentPage < totalPages) handlePageChange(currentPage + 1);
        }}>
          Next
        </a>
      </li>
    );
    
    return items;
  };
  
  // Reset search function
  const resetSearch = (): void => {
    setSearch({
      fromAccount: '',
      toAccount: '',
      transactionType: 'All Types',
      startDate: '',
      endDate: '',
      tokenAmount: '',
      parentHash: '',
      memo: '',
      usePartialMatch: false
    });
  };
  
  return (
    <div className="container my-4">
      <h1 className="mb-4">ICP Ledger Explorer</h1>
      
      <div className="card mb-4">
        <div className="card-body">
          <form onSubmit={handleSubmit}>
            <div className="row">
              <div className="col-md-4">
                <div className="mb-3">
                  <label htmlFor="fromAccount" className="form-label">From Account</label>
                  <input
                    type="text"
                    className="form-control"
                    id="fromAccount"
                    name="fromAccount"
                    value={search.fromAccount}
                    onChange={handleInputChange}
                    placeholder="Sender address"
                  />
                </div>
              </div>
              <div className="col-md-4">
                <div className="mb-3">
                  <label htmlFor="toAccount" className="form-label">To Account</label>
                  <input
                    type="text"
                    className="form-control"
                    id="toAccount"
                    name="toAccount"
                    value={search.toAccount}
                    onChange={handleInputChange}
                    placeholder="Recipient address"
                  />
                </div>
              </div>
              <div className="col-md-4">
                <div className="mb-3">
                  <label htmlFor="transactionType" className="form-label">Transaction Type</label>
                  <select
                    className="form-select"
                    id="transactionType"
                    name="transactionType"
                    value={search.transactionType}
                    onChange={handleInputChange}
                  >
                    {TRANSACTION_TYPES.map(type => (
                      <option key={type} value={type}>{type}</option>
                    ))}
                  </select>
                </div>
              </div>
            </div>
            
            <div className="row">
              <div className="col-md-3">
                <div className="mb-3">
                  <label htmlFor="startDate" className="form-label">Start Date</label>
                  <input
                    type="date"
                    className="form-control"
                    id="startDate"
                    name="startDate"
                    value={search.startDate}
                    onChange={handleInputChange}
                  />
                </div>
              </div>
              <div className="col-md-3">
                <div className="mb-3">
                  <label htmlFor="endDate" className="form-label">End Date</label>
                  <input
                    type="date"
                    className="form-control"
                    id="endDate"
                    name="endDate"
                    value={search.endDate}
                    onChange={handleInputChange}
                  />
                </div>
              </div>
              <div className="col-md-3">
                <div className="mb-3">
                  <label htmlFor="tokenAmount" className="form-label">Min Amount (e8s)</label>
                  <input
                    type="number"
                    className="form-control"
                    id="tokenAmount"
                    name="tokenAmount"
                    value={search.tokenAmount}
                    onChange={handleInputChange}
                    placeholder="Minimum amount"
                  />
                </div>
              </div>
              <div className="col-md-3">
                <div className="mb-3">
                  <label htmlFor="memo" className="form-label">Memo</label>
                  <input
                    type="text"
                    className="form-control"
                    id="memo"
                    name="memo"
                    value={search.memo}
                    onChange={handleInputChange}
                    placeholder="Transaction memo"
                  />
                </div>
              </div>
            </div>
            
            <div className="row">
              <div className="col-md-6">
                <div className="mb-3">
                  <label htmlFor="parentHash" className="form-label">Parent Hash</label>
                  <input
                    type="text"
                    className="form-control"
                    id="parentHash"
                    name="parentHash"
                    value={search.parentHash}
                    onChange={handleInputChange}
                    placeholder="Parent transaction hash"
                  />
                </div>
              </div>
              <div className="col-md-3">
                <div className="form-check mt-4">
                  <input
                    type="checkbox"
                    className="form-check-input"
                    id="usePartialMatch"
                    name="usePartialMatch"
                    checked={search.usePartialMatch}
                    onChange={handleInputChange}
                  />
                  <label className="form-check-label" htmlFor="usePartialMatch">
                    Enable partial address matching
                  </label>
                </div>
              </div>
              <div className="col-md-3 d-flex align-items-end justify-content-end">
                <button type="submit" className="btn btn-primary mb-3">
                  Search
                </button>
                <button 
                  type="button" 
                  className="btn btn-outline-secondary mb-3 ms-2"
                  onClick={resetSearch}
                >
                  Reset
                </button>
              </div>
            </div>
          </form>
        </div>
      </div>
      
      {error && (
        <div className="alert alert-danger">
          Error: {error}
        </div>
      )}
      
      <div className="card">
        <div className="card-body">
          <div className="d-flex justify-content-between align-items-center mb-3">
            <h5 className="mb-0">Results ({totalHits} transactions found)</h5>
            {loading && (
              <div className="spinner-border spinner-border-sm" role="status">
                <span className="visually-hidden">Loading...</span>
              </div>
            )}
          </div>
          
          <div className="table-responsive">
            <table className="table table-striped table-hover">
              <thead>
                <tr>
                  <th>Timestamp</th>
                  <th>Type</th>
                  <th>From</th>
                  <th>To</th>
                  <th>Amount</th>
                  <th>Memo</th>
                  <th>Parent Hash</th>
                </tr>
              </thead>
              <tbody>
                {blocks.length > 0 ? (
                  blocks.map((block, index) => (
                    <tr key={index}>
                      <td>{formatDate(block.timestamp)}</td>
                      <td>{block.transaction?.operation?.type || 'N/A'}</td>
                      <td title={block.transaction?.operation?.from || 'N/A'}>
                        {truncate(block.transaction?.operation?.from)}
                      </td>
                      <td title={block.transaction?.operation?.to || 'N/A'}>
                        {truncate(block.transaction?.operation?.to)}
                      </td>
                      <td>{formatICP(block.transaction?.operation?.amount?.e8s)}</td>
                      <td>{block.transaction?.memo || 'N/A'}</td>
                      <td title={block.parent_hash || 'N/A'}>
                        {truncate(block.parent_hash || '')}
                      </td>
                    </tr>
                  ))
                ) : (
                  <tr>
                    <td colSpan={7} className="text-center">
                      {loading ? 'Loading...' : 'No transactions found'}
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
          
          {totalPages > 1 && (
            <div className="d-flex justify-content-center mt-4">
              <nav aria-label="Transaction pagination">
                <ul className="pagination">
                  {renderPaginationItems()}
                </ul>
              </nav>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}