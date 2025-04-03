/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unused-vars */
import { AccountData, GraphNode, GraphLink, Direction } from "./types";

// Cex -- cex to cex excluded but others should show up
// Defi -- ghost nodes (hidden but in the data)
// Foundation -- foundation to foundation excluded but others should show up
// Identified
// NodeProvider
// Spammer -- excluded completely if tx < 0.1 ICP 
// Sns
// SnsParticipant
// Suspect

const excludedTypes = ["defi", "spammer"];

// Build a graph from an array of AccountData
export function buildGraph(data: AccountData[]): {
  nodes: GraphNode[];
  links: GraphLink[];
} {
  // Build a map of all accounts.
  const allMap = new Map<string, AccountData>();
  data.forEach(acc => {
    allMap.set(acc.account, acc);
  });

  // Build a map of main accounts to be shown in the graph.
  const mainMap = new Map<string, AccountData>();
  data.forEach(acc => {
    if (!excludedTypes.includes(acc.ty.toLowerCase())) {
      mainMap.set(acc.account, acc);
    }
  });

  // Build visible nodeMap from mainMap, but skip those with type "Defi"
  const nodeMap = new Map<string, GraphNode>();
  data.forEach(acc => {
    if (!excludedTypes.includes(acc.ty.toLowerCase())) {
      let extra_info = {};
      // if (acc.account === '296653a4672f7648cd297a3df3147926e16e133b7963fa579f0ee9ab14756dad') {
      //   let newTxs = acc.transactions.sort((a, b) => a.timestamp - b.timestamp);
      //     console.log(newTxs[0].timestamp);
      //     console.log(new Date(newTxs[0].timestamp / 1_000_000).toLocaleString("en-GB"));
      //     console.log(newTxs[newTxs.length - 1].timestamp);
      //     console.log(new Date(newTxs[newTxs.length - 1].timestamp / 1_000_000).toLocaleString("en-GB"));
      // }
      if (acc.transactions && acc.transactions.length > 0) {
      let newTxs = acc.transactions.sort((a, b) => a.timestamp - b.timestamp);
      const total = acc.transactions.reduce((sum, tx) => sum + tx.amount, 0);
      extra_info = {
        average_amount: (total / acc.transactions.length) / 100_000_000,
        tx_count: acc.transactions.length,
        start_date: new Date(newTxs[0].timestamp / 1_000_000).toLocaleString("en-GB"),
        end_date: new Date(newTxs[newTxs.length - 1].timestamp / 1_000_000).toLocaleString("en-GB"),
      };
      }
      nodeMap.set(acc.account, {
        id: acc.account,
        label: acc.name,
        group: acc.ty,
        extra_info: extra_info,
      });
    }
  });

  const connectorMap = new Map<string, Set<string>>();
  const links: GraphLink[] = [];
  
const hiddenTxMap = new Map<string, any[]>(); 
data.forEach(acc => {
  acc.transactions.forEach(tx => {
    if (tx.op_type === "Transfer") {
      const from = tx.from;
      const to = tx.to;
      const fromData = allMap.get(from);
      const toData = allMap.get(to);
      if (
        (fromData && fromData.ty.toLowerCase() === "defi") ||
        (toData && toData.ty.toLowerCase() === "defi")
      ) {
        if (mainMap.has(from)) {
          if (!hiddenTxMap.has(from)) {
            hiddenTxMap.set(from, []);
          }
          hiddenTxMap.get(from)!.push(tx);
        }
        if (mainMap.has(to)) {
          if (!hiddenTxMap.has(to)) {
            hiddenTxMap.set(to, []);
          }
          hiddenTxMap.get(to)!.push(tx);
        }
      }
    }
  });
});
// console.log(hiddenTxMap);
  mainMap.forEach(acc => {
    acc.transactions.forEach(tx => {
      if (tx.op_type === "Transfer") {
        const from = tx.from;
        const to = tx.to;
        const fromIsMain = mainMap.has(from);
        const toIsMain = mainMap.has(to);

        if (fromIsMain && toIsMain) {
          // Both endpoints are main: add direct link (if not already present).
          const existing = links.find(l =>
            (l.source === from && l.target === to) ||
            (l.source === to && l.target === from)
          );
          if (!existing) {
            const direction: Direction = (from === acc.account) ? Direction.SEND : Direction.RECEIVE;
            links.push({
              source: from,
              target: to,
              direction,
            });
          } else {
            const direction: Direction = (from === acc.account) ? Direction.SEND : Direction.RECEIVE;
            if (existing.direction !== direction) {
              existing.direction = Direction.BOTH;
            }
          }
        } else {
          // At least one endpoint is extra.
          // Record extra account(s) and the main account from the current AccountData.
          if (!fromIsMain) {
            if (!connectorMap.has(from)) {
              connectorMap.set(from, new Set());
            }
            connectorMap.get(from)!.add(acc.account);
          }
          if (!toIsMain) {
            if (!connectorMap.has(to)) {
              connectorMap.set(to, new Set());
            }
            connectorMap.get(to)!.add(acc.account);
          }
          // Do not add a link here yet.
        }
      }
    });
  });

  const connectorGroupMap = new Map<string, { extraIds: Set<string>; mainSet: Set<string> }>();

  connectorMap.forEach((mainSet, extraId) => {
    if (mainSet.size > 1) {
      // Check if all connected main accounts are Exchange.
      const allExchange = Array.from(mainSet).every(mainAccId => {
        const mainData = mainMap.get(mainAccId);
        return mainData && mainData.ty === "Cex";
      });
      // Check if all connected main accounts are Foundation.
      const allFoundation = Array.from(mainSet).every(mainAccId => {
        const mainData = mainMap.get(mainAccId);
        return mainData && mainData.ty === "Foundation";
      });
      // Only create the connector node if NOT all are Exchange or all are Foundation.
      if (!(allExchange || allFoundation)) {
        let label = "";
        mainSet.forEach(mainAccId => {
          const mainData = mainMap.get(mainAccId);
          if (mainData) {
            label += initials(mainData.name);
          }
        });
        if (connectorGroupMap.has(label)) {
          // Merge: add the extraId and union the mainSet.
          const existing = connectorGroupMap.get(label)!;
          existing.extraIds.add(extraId);
          mainSet.forEach(id => existing.mainSet.add(id));
        } else {
          connectorGroupMap.set(label, { extraIds: new Set([extraId]), mainSet: new Set(mainSet) });
        }
      }
    }
  });
  // Create one connector node for each group in connectorGroupMap.
  connectorGroupMap.forEach(({ extraIds, mainSet }, label) => {
    // Here we use the label as the id of the connector node.
    // Also attach a new property 'mainAccounts' (or similar) to store the main account IDs.
    nodeMap.set(label, {
      id: label,
      label,
      group: "connector",
      mainAccounts: Array.from(mainSet)  // <-- extra property for later use
    });
    // Create a link from each connected main node to this connector node.
    mainSet.forEach(mainAccId => {
      links.push({
        source: mainAccId,
        target: label,
        direction: Direction.SEND, // Adjust if needed
      });
    });
  });

  nodeMap.forEach((node, id) => {
    node.defiTxs = hiddenTxMap.get(id) || [];
  });

  const nodes = Array.from(nodeMap.values());
  
  return { nodes, links };
}

function initials(name: string): string {
  return name.replace(/\s+/g, "").slice(0, 2);
}
