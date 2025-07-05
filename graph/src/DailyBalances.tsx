import React, { useEffect, useState } from 'react';
import { 
  select, 
  scaleLinear, 
  scaleOrdinal, 
  schemeCategory10, 
  axisBottom, 
  axisLeft, 
  line, 
  area,
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
  const [selectedDay, setSelectedDay] = useState<number | null>(null);
  const [dayBalances, setDayBalances] = useState<{[address: string]: number}>({});

  useEffect(() => {
    async function loadData() {
      try {
        const response = await fetch('/daily_balances.json');
        const balanceData = await response.json();
        setData(balanceData);
        
        // Select all addresses for initial display
        setSelectedAddresses(Object.keys(balanceData));
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
    select('#cumulative-chart').selectAll('*').remove();

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
          return timeFormat('%m/%d/%y')(date);
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
          return timeFormat('%m/%d/%y')(date);
        }));

      combinedG.append('g')
        .call(axisLeft(combinedYScale).tickFormat((d: any) => {
          const icpValue = (d as number) / 100_000_000;
          if (icpValue >= 1_000_000) {
            return `${(icpValue / 1_000_000).toFixed(1)}M`;
          } else if (icpValue >= 1_000) {
            return `${(icpValue / 1_000).toFixed(0)}K`;
          } else {
            return `${icpValue.toFixed(0)}`;
          }
        }));

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

    // Create cumulative total chart
    if (data) {
      const cumulativeContainer = select('#cumulative-chart')
        .append('div')
        .attr('class', 'mb-4')
        .style('border', '1px solid #dee2e6')
        .style('border-radius', '0.375rem')
        .style('padding', '1rem');
      
      cumulativeContainer
        .append('h5')
        .text('Cumulative Total - All Pattern Addresses');
      
      const cumulativeSvg = cumulativeContainer
        .append('svg')
        .attr('width', width + margin.left + margin.right)
        .attr('height', height + margin.top + margin.bottom);

      const cumulativeG = cumulativeSvg.append('g')
        .attr('transform', `translate(${margin.left},${margin.top})`);

      // Calculate cumulative totals per day
      const cumulativeTotals: [number, number][] = [];
      const allDays = new Set<number>();
      
      // Get all unique days
      Object.values(data).forEach(addressData => {
        addressData.forEach(([day]) => allDays.add(day));
      });
      
      // Sort days and calculate totals
      const sortedDays = Array.from(allDays).sort((a, b) => a - b);
      
      sortedDays.forEach(day => {
        let dayTotal = 0;
        Object.keys(data).forEach(address => {
          dayTotal += getBalanceForDay(address, day);
        });
        cumulativeTotals.push([day, dayTotal]);
      });

      // Create scales for cumulative chart
      const cumulativeXScale = scaleLinear()
        .domain(extent(cumulativeTotals, (d: [number, number]) => d[0]) as [number, number])
        .range([0, width]);

      const cumulativeYScale = scaleLinear()
        .domain([0, max(cumulativeTotals, (d: [number, number]) => d[1]) as number])
        .range([height, 0]);

      // Add axes
      cumulativeG.append('g')
        .attr('transform', `translate(0,${height})`)
        .call(axisBottom(cumulativeXScale).tickFormat((d: any) => {
          const date = new Date((d as number) * 24 * 60 * 60 * 1000);
          return timeFormat('%m/%d/%y')(date);
        }));

      cumulativeG.append('g')
        .call(axisLeft(cumulativeYScale).tickFormat((d: any) => {
          const icpValue = (d as number) / 100_000_000;
          if (icpValue >= 1_000_000) {
            return `${(icpValue / 1_000_000).toFixed(1)}M`;
          } else if (icpValue >= 1_000) {
            return `${(icpValue / 1_000).toFixed(0)}K`;
          } else {
            return `${icpValue.toFixed(0)}`;
          }
        }));

      // Add axis labels
      cumulativeG.append('text')
        .attr('transform', 'rotate(-90)')
        .attr('y', 0 - margin.left)
        .attr('x', 0 - (height / 2))
        .attr('dy', '1em')
        .style('text-anchor', 'middle')
        .style('font-size', '12px')
        .text('Total Balance (ICP)');

      cumulativeG.append('text')
        .attr('transform', `translate(${width / 2}, ${height + 40})`)
        .style('text-anchor', 'middle')
        .style('font-size', '12px')
        .text('Date');

      // Create line generator for cumulative chart
      const cumulativeLineGenerator = line<[number, number]>()
        .x((d: [number, number]) => cumulativeXScale(d[0]))
        .y((d: [number, number]) => cumulativeYScale(d[1]))
        .curve(curveMonotoneX);

      // Draw the cumulative line
      cumulativeG.append('path')
        .datum(cumulativeTotals)
        .attr('fill', 'none')
        .attr('stroke', '#28a745')
        .attr('stroke-width', 3)
        .attr('d', cumulativeLineGenerator);

      // Add area fill
      const areaGenerator = area<[number, number]>()
        .x((d: [number, number]) => cumulativeXScale(d[0]))
        .y0(height)
        .y1((d: [number, number]) => cumulativeYScale(d[1]))
        .curve(curveMonotoneX);

      cumulativeG.append('path')
        .datum(cumulativeTotals)
        .attr('fill', '#28a745')
        .attr('fill-opacity', 0.2)
        .attr('d', areaGenerator);

      // Add dots for data points
      cumulativeG.selectAll('.cumulative-dot')
        .data(cumulativeTotals)
        .enter().append('circle')
        .attr('class', 'cumulative-dot')
        .attr('cx', (d: [number, number]) => cumulativeXScale(d[0]))
        .attr('cy', (d: [number, number]) => cumulativeYScale(d[1]))
        .attr('r', 2)
        .attr('fill', '#28a745');

      // Add current total display
      const currentTotal = cumulativeTotals[cumulativeTotals.length - 1]?.[1] || 0;
      cumulativeContainer
        .append('div')
        .attr('class', 'mt-3')
        .style('text-align', 'center')
        .html(`<h6>Current Total: <span class="text-success">${(currentTotal / 100_000_000).toFixed(2)} ICP</span></h6>`);
    }

  }, [data, selectedAddresses]);

  const filteredAddresses = data ? Object.keys(data).filter(address => 
    address.toLowerCase().includes(searchTerm.toLowerCase())
  ) : [];

  const toggleAddress = (address: string) => {
    setSelectedAddresses(prev => 
      prev.includes(address) 
        ? prev.filter(a => a !== address)
        : [...prev, address] // No limit on addresses
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

  // Function to get balance for a specific day
  const getBalanceForDay = (address: string, day: number): number => {
    if (!data || !data[address]) return 0;
    
    const addressData = data[address];
    // Find the exact day or the most recent day before it
    let balance = 0;
    for (const [dataDay, dataBalance] of addressData) {
      if (dataDay <= day) {
        balance = dataBalance;
      } else {
        break;
      }
    }
    return balance;
  };

  // Function to handle day selection
  const handleDaySelection = (day: number) => {
    setSelectedDay(day);
    const balances: {[address: string]: number} = {};
    
    selectedAddresses.forEach(address => {
      balances[address] = getBalanceForDay(address, day);
    });
    
    setDayBalances(balances);
  };

  // Function to handle date selection from calendar
  const handleDateSelection = (dateString: string) => {
    const selectedDate = new Date(dateString);
    const dayNumber = Math.floor(selectedDate.getTime() / (24 * 60 * 60 * 1000));
    handleDaySelection(dayNumber);
  };

  // Convert day number to date string for calendar input
  const dayToDateString = (day: number): string => {
    const date = new Date(day * 24 * 60 * 60 * 1000);
    return date.toISOString().split('T')[0];
  };

  // Format date as DD-MM-YYYY
  const formatDateDDMMYYYY = (day: number): string => {
    const date = new Date(day * 24 * 60 * 60 * 1000);
    const dd = String(date.getDate()).padStart(2, '0');
    const mm = String(date.getMonth() + 1).padStart(2, '0');
    const yyyy = date.getFullYear();
    return `${dd}-${mm}-${yyyy}`;
  };

  // Get available date range
  const getDateRange = () => {
    if (!data) return { minDay: 0, maxDay: 0 };
    
    let minDay = Infinity;
    let maxDay = -Infinity;
    
    Object.values(data).forEach(addressData => {
      addressData.forEach(([day]) => {
        minDay = Math.min(minDay, day);
        maxDay = Math.max(maxDay, day);
      });
    });
    
    return { minDay, maxDay };
  };

  const { minDay, maxDay } = getDateRange();

  return (
    <div className="container mt-4">
      <div className="row">
        <div className="col-12">
          <h2>Daily Balance Chart for Pattern Addresses</h2>
          <p className="text-muted">
            Track balance changes over time for pattern addresses. Select any number of addresses to compare.
          </p>
        </div>
      </div>

      {/* Day Selection Section */}
      <div className="row mb-4">
        <div className="col-md-6">
          <div className="card">
            <div className="card-header">
              <h5>Select Specific Date</h5>
            </div>
            <div className="card-body">
              <div className="mb-3">
                <label htmlFor="dateSelect" className="form-label">Choose a date to view balances:</label>
                <input
                  type="date"
                  className="form-control"
                  id="dateSelect"
                  min={dayToDateString(minDay)}
                  max={dayToDateString(maxDay)}
                  value={selectedDay ? dayToDateString(selectedDay) : dayToDateString(maxDay)}
                  onChange={(e) => handleDateSelection(e.target.value)}
                />
                <div className="form-text">
                  Available date range: {formatDateDDMMYYYY(minDay)} - {formatDateDDMMYYYY(maxDay)}
                </div>
              </div>
              {selectedDay && (
                <div className="alert alert-info">
                  <strong>Selected Date:</strong> {formatDateDDMMYYYY(selectedDay)}
                </div>
              )}
            </div>
          </div>
        </div>
        
        {selectedDay && Object.keys(dayBalances).length > 0 && (
          <div className="col-md-6">
            <div className="card">
              <div className="card-header">
                <h5>Total Balance on Selected Day</h5>
              </div>
              <div className="card-body">
                <h3 className="text-primary">
                  {(Object.values(dayBalances).reduce((sum, balance) => sum + balance, 0) / 100_000_000).toFixed(2)} ICP
                </h3>
                <p className="text-muted">
                  Total across {Object.keys(dayBalances).length} selected addresses
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Individual Address Balances for Selected Day */}
      {selectedDay && Object.keys(dayBalances).length > 0 && (
        <div className="row mb-4">
          <div className="col-12">
            <div className="card">
              <div className="card-header">
                <h5>Individual Address Balances on {formatDateDDMMYYYY(selectedDay)}</h5>
              </div>
              <div className="card-body">
                <div className="row">
                  {Object.entries(dayBalances)
                    .sort(([,a], [,b]) => b - a) // Sort by balance descending
                    .map(([address, balance]) => (
                    <div key={address} className="col-md-4 col-lg-3 mb-3">
                      <div className="card h-100">
                        <div className="card-body">
                          <h6 className="card-title">{address.substring(0, 12)}...</h6>
                          <p className="card-text">
                            <strong>{(balance / 100_000_000).toFixed(2)} ICP</strong>
                          </p>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      <div className="row">
        <div className="col-12">
          <h3>Charts</h3>
        </div>
      </div>

      <div className="row">
        <div className="col-md-12">
          <div className="card">
            <div className="card-header">
              <h5>Cumulative Total</h5>
            </div>
            <div className="card-body">
              <div id="cumulative-chart"></div>
            </div>
          </div>
        </div>
      </div>

      <div className="row mt-4">
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
              <h5>Select Addresses ({selectedAddresses.length})</h5>
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
                  className="btn btn-sm btn-outline-primary me-2"
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
                <button 
                  className="btn btn-sm btn-outline-success"
                  onClick={() => {
                    if (data) {
                      setSelectedAddresses(Object.keys(data));
                    }
                  }}
                >
                  Select All
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