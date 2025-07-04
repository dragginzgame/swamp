import React, { useEffect, useState } from 'react';
import { 
  select, 
  scaleLinear, 
  scaleOrdinal, 
  schemeCategory10, 
  axisBottom, 
  axisLeft, 
  line, 
  curveMonotoneX, 
  extent, 
  max, 
  timeFormat
} from 'd3';

interface DailyBalanceData {
  [address: string]: [number, number][]; // [day, balance] pairs
}

export const DailyBalances: React.FC = () => {
  const [data, setData] = useState<DailyBalanceData | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedAddresses, setSelectedAddresses] = useState<string[]>([]);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    async function loadData() {
      try {
        const response = await fetch('/daily_balances.json');
        const balanceData = await response.json();
        setData(balanceData);
        
        // Select top 10 addresses by max balance for initial display
        const addressesWithMaxBalance = Object.entries(balanceData).map(([address, balances]) => ({
          address,
          maxBalance: Math.max(...(balances as [number, number][]).map(([_, balance]) => balance))
        }));
        
        addressesWithMaxBalance.sort((a, b) => b.maxBalance - a.maxBalance);
        setSelectedAddresses(addressesWithMaxBalance.slice(0, 10).map(item => item.address));
      } catch (error) {
        console.error('Failed to load daily balance data:', error);
      } finally {
        setLoading(false);
      }
    }
    
    loadData();
  }, []);

  useEffect(() => {
    if (!data || selectedAddresses.length === 0) return;

    // Clear previous charts
    select('#balance-charts').selectAll('*').remove();
    select('#combined-chart').selectAll('*').remove();

    const margin = { top: 20, right: 20, bottom: 60, left: 80 };
    const width = 800 - margin.left - margin.right;
    const height = 300 - margin.top - margin.bottom;

    // Create individual chart for each selected address
    selectedAddresses.forEach((address, index) => {
      if (!data[address]) return;
      
      const addressData = data[address];
      
      // Create container for this chart
      const chartContainer = select('#balance-charts')
        .append('div')
        .attr('class', 'mb-4')
        .style('border', '1px solid #dee2e6')
        .style('border-radius', '0.375rem')
        .style('padding', '1rem');
      
      // Add title
      chartContainer
        .append('h5')
        .text(`${address}`);
      
      const svg = chartContainer
        .append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom);

      const g = svg.append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);

      // Create scales for this address
      const xScale = scaleLinear()
        .domain(extent(addressData, (d: [number, number]) => d[0]) as [number, number])
        .range([0, width]);

      const yScale = scaleLinear()
        .domain([0, max(addressData, (d: [number, number]) => d[1]) as number])
        .range([height, 0]);

      // Add axes
      g.append('g')
        .attr('transform', `translate(0,${height})`)
        .call(axisBottom(xScale).tickFormat((d: any) => {
          // Convert day number back to date
          const date = new Date((d as number) * 24 * 60 * 60 * 1000);
          return timeFormat('%m/%d')(date);
        }));

      g.append('g')
        .call(axisLeft(yScale).tickFormat((d: any) => `${(d as number / 100_000_000).toFixed(1)}`));

      // Add axis labels
      g.append('text')
        .attr('transform', 'rotate(-90)')
        .attr('y', 0 - margin.left)
        .attr('x', 0 - (height / 2))
        .attr('dy', '1em')
        .style('text-anchor', 'middle')
        .style('font-size', '12px')
        .text('Balance (ICP)');

      g.append('text')
        .attr('transform', `translate(${width / 2}, ${height + 40})`)
        .style('text-anchor', 'middle')
        .style('font-size', '12px')
        .text('Date');

      // Create line generator
      const lineGenerator = line<[number, number]>()
        .x((d: [number, number]) => xScale(d[0]))
        .y((d: [number, number]) => yScale(d[1]))
        .curve(curveMonotoneX);

      // Draw line for this address
      g.append('path')
        .datum(addressData)
        .attr('fill', 'none')
        .attr('stroke', '#007bff')
        .attr('stroke-width', 2)
        .attr('d', lineGenerator);

      // Add dots for data points
      g.selectAll('.dot')
        .data(addressData)
        .enter().append('circle')
        .attr('class', 'dot')
        .attr('cx', (d: [number, number]) => xScale(d[0]))
        .attr('cy', (d: [number, number]) => yScale(d[1]))
        .attr('r', 2)
        .attr('fill', '#007bff');
    });

    // Create combined chart
    if (selectedAddresses.length > 1) {
      const combinedContainer = select('#combined-chart')
        .append('div')
        .attr('class', 'mb-4')
        .style('border', '1px solid #dee2e6')
        .style('border-radius', '0.375rem')
        .style('padding', '1rem');
      
      combinedContainer
        .append('h5')
        .text('Combined View - All Selected Addresses');
      
      const combinedSvg = combinedContainer
        .append('svg')
        .attr('width', width + margin.left + margin.right + 150) // Extra space for legend
        .attr('height', height + margin.top + margin.bottom);

      const combinedG = combinedSvg.append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);

      // Get all data points for selected addresses
      const allData: Array<{address: string, day: number, balance: number}> = [];
      selectedAddresses.forEach(address => {
        if (data[address]) {
          data[address].forEach(([day, balance]) => {
            allData.push({ address, day, balance });
          });
        }
      });

      // Create scales for combined chart
      const combinedXScale = scaleLinear()
        .domain(extent(allData, (d: {address: string, day: number, balance: number}) => d.day) as [number, number])
        .range([0, width]);

      const combinedYScale = scaleLinear()
        .domain([0, max(allData, (d: {address: string, day: number, balance: number}) => d.balance) as number])
        .range([height, 0]);

      const colorScale = scaleOrdinal(schemeCategory10);

      // Add axes
      combinedG.append('g')
        .attr('transform', `translate(0,${height})`)
        .call(axisBottom(combinedXScale).tickFormat((d: any) => {
          const date = new Date((d as number) * 24 * 60 * 60 * 1000);
          return timeFormat('%m/%d')(date);
        }));

      combinedG.append('g')
        .call(axisLeft(combinedYScale).tickFormat((d: any) => `${(d as number / 100_000_000).toFixed(1)}`));

      // Add axis labels
      combinedG.append('text')
        .attr('transform', 'rotate(-90)')
        .attr('y', 0 - margin.left)
        .attr('x', 0 - (height / 2))
        .attr('dy', '1em')
        .style('text-anchor', 'middle')
        .style('font-size', '12px')
        .text('Balance (ICP)');

      combinedG.append('text')
        .attr('transform', `translate(${width / 2}, ${height + 40})`)
        .style('text-anchor', 'middle')
        .style('font-size', '12px')
        .text('Date');

      // Create line generator for combined chart
      const combinedLineGenerator = line<[number, number]>()
        .x((d: [number, number]) => combinedXScale(d[0]))
        .y((d: [number, number]) => combinedYScale(d[1]))
        .curve(curveMonotoneX);

      // Draw lines for each selected address
      selectedAddresses.forEach((address, index) => {
        if (data[address]) {
          const color = colorScale(address);
          
          combinedG.append('path')
            .datum(data[address])
            .attr('fill', 'none')
            .attr('stroke', color)
            .attr('stroke-width', 2)
            .attr('d', combinedLineGenerator)
            .style('opacity', 0.8);

          // Add legend
          combinedG.append('circle')
            .attr('cx', width + 20)
            .attr('cy', index * 20 + 10)
            .attr('r', 4)
            .attr('fill', color);

          combinedG.append('text')
            .attr('x', width + 30)
            .attr('y', index * 20 + 15)
            .attr('fill', color)
            .style('font-size', '11px')
            .text(`${address.substring(0, 12)}...`);
        }
      });
    }

  }, [data, selectedAddresses]);

  const filteredAddresses = data ? Object.keys(data).filter(address => 
    address.toLowerCase().includes(searchTerm.toLowerCase())
  ) : [];

  const toggleAddress = (address: string) => {
    setSelectedAddresses(prev => 
      prev.includes(address) 
        ? prev.filter(a => a !== address)
        : [...prev, address].slice(-10) // Limit to 10 addresses
    );
  };

  if (loading) {
    return (
      <div className="container mt-4">
        <div className="text-center">
          <div className="spinner-border" role="status">
            <span className="visually-hidden">Loading...</span>
          </div>
          <p className="mt-2">Loading daily balance data...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container mt-4">
      <div className="row">
        <div className="col-12">
          <h2>Daily Balance Chart for Pattern Addresses</h2>
          <p className="text-muted">
            Track balance changes over time for pattern addresses. Select up to 10 addresses to compare.
          </p>
        </div>
      </div>

      <div className="row">
        <div className="col-md-12">
          <div className="card">
            <div className="card-header">
              <h5>Combined View</h5>
            </div>
            <div className="card-body">
              <div id="combined-chart"></div>
            </div>
          </div>
        </div>
      </div>

      <div className="row mt-4">
        <div className="col-md-12">
          <div className="card">
            <div className="card-header">
              <h5>Individual Address Charts</h5>
            </div>
            <div className="card-body">
              <div id="balance-charts"></div>
            </div>
          </div>
        </div>

        <div className="col-md-4">
          <div className="card">
            <div className="card-header">
              <h5>Select Addresses ({selectedAddresses.length}/10)</h5>
            </div>
            <div className="card-body">
              <div className="mb-3">
                <input
                  type="text"
                  className="form-control"
                  placeholder="Search addresses..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                />
              </div>
              
              <div className="mb-3">
                <button 
                  className="btn btn-sm btn-outline-secondary me-2"
                  onClick={() => setSelectedAddresses([])}
                >
                  Clear All
                </button>
                <button 
                  className="btn btn-sm btn-outline-primary"
                  onClick={() => {
                    if (data) {
                      const top10 = Object.entries(data)
                        .map(([address, balances]) => ({
                          address,
                          maxBalance: Math.max(...(balances as [number, number][]).map(([_, balance]) => balance))
                        }))
                        .sort((a, b) => b.maxBalance - a.maxBalance)
                        .slice(0, 10)
                        .map(item => item.address);
                      setSelectedAddresses(top10);
                    }
                  }}
                >
                  Top 10
                </button>
              </div>

              <div className="address-list" style={{maxHeight: '400px', overflowY: 'auto'}}>
                {filteredAddresses.map(address => {
                  const isSelected = selectedAddresses.includes(address);
                  const maxBalance = data && data[address] 
                    ? Math.max(...data[address].map(([_, balance]) => balance)) / 100_000_000
                    : 0;
                  
                  return (
                    <div 
                      key={address} 
                      className={`card mb-2 ${isSelected ? 'border-primary' : ''}`}
                      style={{cursor: 'pointer'}}
                      onClick={() => toggleAddress(address)}
                    >
                      <div className="card-body py-2">
                        <div className="form-check">
                          <input 
                            className="form-check-input" 
                            type="checkbox" 
                            checked={isSelected}
                            readOnly
                          />
                          <label className="form-check-label">
                            <small>
                              <strong>{address.substring(0, 8)}...</strong><br/>
                              Max: {maxBalance.toFixed(1)} ICP
                            </small>
                          </label>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </div>
      </div>

      {selectedAddresses.length > 0 && (
        <div className="row mt-4">
          <div className="col-12">
            <div className="card">
              <div className="card-header">
                <h5>Selected Addresses Summary</h5>
              </div>
              <div className="card-body">
                <div className="row">
                  {selectedAddresses.map(address => {
                    const addressData = data?.[address];
                    if (!addressData) return null;
                    
                    const currentBalance = addressData[addressData.length - 1]?.[1] || 0;
                    const maxBalance = Math.max(...addressData.map(([_, balance]) => balance));
                    
                    return (
                      <div key={address} className="col-md-6 col-lg-4 mb-3">
                        <div className="card h-100">
                          <div className="card-body">
                            <h6 className="card-title">{address}</h6>
                            <p className="card-text">
                              <small>
                                Current: {(currentBalance / 100_000_000).toFixed(1)} ICP<br/>
                                Peak: {(maxBalance / 100_000_000).toFixed(1)} ICP
                              </small>
                            </p>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};