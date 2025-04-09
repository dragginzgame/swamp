"use client";
import { SimulationNodeDatum, SimulationLinkDatum } from "d3-force";

export interface AccountData {
    name: string;
    principal?: string;
    account: string;
    ty: string;
    extra_accounts: string[];
    transactions: Transaction[];
  }
  
  export interface Transaction {
    op_type: Operation;
    to: string;
    from: string;
    id: number;
    amount: number;
    timestamp: number;
  }
  
  export type Operation =
    | "Transfer"
    | "Mint"
    | "Burn"
    | "Approve"
  
  // D3 simulation node & link definitions
  
  export interface GraphNode extends SimulationNodeDatum {
    id: string;    // matches the "account" hex string
    label: string; // e.g. the name
    group: string; // e.g. "Exchange", "Individual"
    color?: string;
    mainAccounts?: string[];
    defiTxs?: Transaction[];
    extra_info?: any;
  }
  
  export interface GraphLink extends SimulationLinkDatum<GraphNode> {
    source: string; // account "from"
    target: string; // account "to"
    direction?: Direction;
    connectors?: string[];
  }


export enum Direction {
  SEND = "SEND",
  RECEIVE = "RECEIVE",
  BOTH = "BOTH",
}

// Each node extends SimulationNodeDatum
export interface Node extends SimulationNodeDatum {
  id: string;
  account: string;
  color?: string;
}

// Each link extends SimulationLinkDatum<Node>
export interface Link extends SimulationLinkDatum<Node> {
  direction: Direction;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[]; 
};
