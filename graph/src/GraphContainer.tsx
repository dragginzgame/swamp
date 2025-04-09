/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unused-vars */
import React, { useEffect, useState } from "react";
import Graph from "./Graph";
import { AccountData, GraphNode, GraphData } from "./types"; // adjust the path as needed
import Select from 'react-select'

interface GraphContainerProps {
  data: GraphData;
  width: number;
  height: number;
  onNodeClick: (node: GraphNode) => void;
  onLinkClick?: (data: any) => void;
  loading: boolean;
  highlightNodeId?: string;
}

export function GraphContainer({
  data,
  width,
  height,
  onNodeClick,
  onLinkClick,
  loading,
}: GraphContainerProps) {
  const [highlightNodeId, setHighlightNodeId] = useState<string | undefined>(undefined);
  const [selectData, setSelectData] = useState<any[]>([]);

  useEffect(() => {
    const selectValues = [...data.nodes].map((node) => {
      return { value: node.id, label: node.label };
    });
    setSelectData(selectValues);
  }
  , []);
  const handleSearch = (selectedOption: any) => {
    if (selectedOption) {
      // Use the account id as the node id.
      setHighlightNodeId(selectedOption.value);
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

  return (
    <div className="graph-container">
      <Select options={selectData} placeholder="Find node..." onChange={handleSearch} />
      <br />
      <Graph
        data={data}
        width={width}
        height={height}
        onNodeClick={onNodeClick}
        onLinkClick={onLinkClick}
        highlightNodeId={highlightNodeId}
      />
    </div>
  );
}