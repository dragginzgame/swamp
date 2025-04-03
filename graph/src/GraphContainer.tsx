/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unused-vars */
import React, { useState } from "react";
import Graph from "./Graph";
import { AccountData, GraphNode } from "./types"; // adjust the path as needed
import { buildGraph } from "./graphData";
import Select from 'react-select'

interface GraphContainerProps {
  data: AccountData[];
  width: number;
  height: number;
  onNodeClick: (node: GraphNode) => void;
  loading: boolean;
  highlightNodeId?: string;
}

export function GraphContainer({
  data,
  width,
  height,
  onNodeClick,
  loading,
}: GraphContainerProps) {
  const [highlightNodeId, setHighlightNodeId] = useState<string | undefined>(undefined);

  const handleSearch = (selectedOption: any) => {
    // console.log(selectedOption);
    const matchingEntry = data.find((entry) =>
      entry.name.toLowerCase().includes(selectedOption.label.toLowerCase())
    );
    // console.log(matchingEntry);
    if (matchingEntry) {
      // Use the account id as the node id.
      setHighlightNodeId(matchingEntry.account);
    } else {
      setHighlightNodeId(undefined);
    }
  };
  if (loading) {
    return (
      <div
        className="d-flex justify-content-center align-items-center"
        style={{ height: "100vh" }}
      >
        <div className="spinner-border" role="status">
          <span className="visually-hidden">Loading...</span>
        </div>
      </div>
    );
  }
  // 
  const { nodes } = buildGraph(data);
  const selectValues = [...nodes].map((node) => {
    return { value: node.id, label: node.label };
  });
  return (
    <div className="graph-container">
      <Select options={selectValues} placeholder="Find node..." onChange={handleSearch} />
      <br />
      <Graph
        data={data}
        width={width}
        height={height}
        onNodeClick={onNodeClick}
        highlightNodeId={highlightNodeId}
      />
    </div>
  );
}