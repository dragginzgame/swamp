/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unused-vars */
import { AccountData, GraphNode, GraphLink, Direction } from "./types";
// Defi -- (NODE) ghost nodes (hidden but in the data) 
// Cex -- (LINK) cex to cex excluded but others should show up
// Foundation -- (LINK) foundation to foundation excluded but others should show up
// Spammer -- (LINK) excluded completely if tx < 0.1 ICP 
// Identified
// NodeProvider
// Sns
// SnsParticipant
// Suspect


// Build a graph from an array of AccountData
export function buildGraph(data: AccountData[]): {
  nodes: GraphNode[];
  links: GraphLink[];
} {
  // Build a map of all accounts including defi cause we need the transactions 
  const allMap = new Map<string, AccountData>();
  const accountToMain = new Map<string, string>();

  data.forEach(acc => {
    allMap.set(acc.account, acc);
    accountToMain.set(acc.account, acc.account);
    acc.extra_accounts.forEach(extra => {
      accountToMain.set(extra, acc.account);
    });
  });

  const nodeMap = new Map<string, GraphNode>();
  // --- main nodes except defi ---
  data.forEach(acc => {
    if (acc.ty.toLowerCase() !== 'defi') {
      let extra_info = {};
      if (acc.transactions && acc.transactions.length > 0) {
        let newTxs = acc.transactions.sort((a, b) => a.timestamp - b.timestamp);
        const total = acc.transactions.reduce((sum, tx) => sum + tx.amount, 0);
        extra_info = {
          average_amount: (total / acc.transactions.length) / 100_000_000,
          tx_count: acc.transactions.length,
          start_date: new Date(newTxs[0].timestamp / 1_000_000).toLocaleString("en-GB"),
          end_date: new Date(newTxs[newTxs.length - 1].timestamp / 1_000_000).toLocaleString("en-GB"),
          extra_accounts: acc.extra_accounts
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
  console.log(nodeMap.has("4dfa8f7797f1bb03223abd9a9bba306d79a755d43a3dd7ec15220cbbc38ce8af"));
  const links: GraphLink[] = [];
  const hiddenLinksMap = new Map<string, GraphLink[]>();
  // --- main Link creation between known nodes ---
  allMap.forEach((acc: AccountData) => {
    acc.transactions.forEach(tx => {
      if (tx.op_type !== "Transfer") return;

      const fromMain = accountToMain.get(tx.from);
      const toMain = accountToMain.get(tx.to);

      // Skip if same root node
      if (!fromMain || !toMain || fromMain === toMain) return;

      const fromNode = allMap.get(fromMain);
      const toNode = allMap.get(toMain);
      if (!fromNode || !toNode) return;

      const fromTy = fromNode.ty.toLowerCase();
      const toTy = toNode.ty.toLowerCase();

      const direction: Direction = (fromMain === acc.account) ? Direction.SEND : Direction.RECEIVE;

      const link: GraphLink = {
        source: fromMain,
        target: toMain,
        direction
      };

      // If either node is defi, store it in the hidden map
      if (fromTy === "defi" || toTy === "defi") {
        if (fromTy !== "defi" && nodeMap.has(fromMain)) {
          if (!hiddenLinksMap.has(fromMain)) hiddenLinksMap.set(fromMain, []);
          hiddenLinksMap.get(fromMain)!.push(link);
        }
        if (toTy !== "defi" && nodeMap.has(toMain)) {
          if (!hiddenLinksMap.has(toMain)) hiddenLinksMap.set(toMain, []);
          hiddenLinksMap.get(toMain)!.push(link);
        }
        return; // Skip visible graph link creation
      }

      // Exclude CEX <-> CEX or Foundation <-> Foundation
      if ((fromTy === "cex" && toTy === "cex") || (fromTy === "foundation" && toTy === "foundation")) return;

      // Exclude low-value spammer txs
      if (fromTy === "spammer" && tx.amount < 10_000_000) return;

      const existing = links.find(l =>
        (l.source === fromMain && l.target === toMain) ||
        (l.source === toMain && l.target === fromMain)
      );


      if (!existing) {
        links.push(link);
      } else if (existing.direction !== direction) {
        existing.direction = Direction.BOTH;
      }
    });
  });

// --- Connector Node Detection ---
// Build a mapping from external account IDs (the connector) to the set of main accounts that used them.
const connectorNodeMap = new Map<string, Set<string>>();

allMap.forEach((acc: AccountData) => {
  acc.transactions.forEach(tx => {
    if (tx.op_type !== "Transfer") return;

    const fromMain = accountToMain.get(tx.from);
    const toMain = accountToMain.get(tx.to);

    // Skip internal transfers (i.e. same root/main account)
    if (fromMain && toMain && fromMain === toMain) return;

    // Determine if we know the main account for each end of the transaction.
    const fromIsKnown = fromMain && nodeMap.has(fromMain);
    const toIsKnown = toMain && nodeMap.has(toMain);

    // Branch 1: Known fromMain and external connector is tx.to.
    if (fromIsKnown && !toMain) {
      let addConnector = true;
      // Extra check: if the known main account (fromMain) is flagged as spammer, skip it.
      if (allMap.has(fromMain)) {
        const mainAcc = allMap.get(fromMain)!;
        if (mainAcc.ty.toLowerCase() === "spammer") {
          addConnector = false;
        }
      }
      // Check the external account at tx.to.
      if (allMap.has(tx.to)) {
        const externalAcc = allMap.get(tx.to)!;
        // If the external account is flagged as spammer and the transaction amount is <= threshold, skip.
        if (externalAcc.ty.toLowerCase() === "spammer" && tx.amount <= 1000000) {
          addConnector = false;
        }
      }
      if (addConnector) {
        if (!connectorNodeMap.has(tx.to)) connectorNodeMap.set(tx.to, new Set());
        connectorNodeMap.get(tx.to)!.add(fromMain);
      }
    }

    // Branch 2: Known toMain and external connector is tx.from.
    if (toIsKnown && !fromMain) {
      let addConnector = true;
      // Extra check: if the known main account (toMain) is flagged as spammer, skip it.
      if (allMap.has(toMain)) {
        const mainAcc = allMap.get(toMain)!;
        if (mainAcc.ty.toLowerCase() === "spammer") {
          addConnector = false;
        }
      }
      // Check the external account at tx.from.
      if (allMap.has(tx.from)) {
        const externalAcc = allMap.get(tx.from)!;
        if (externalAcc.ty.toLowerCase() === "spammer" && tx.amount <= 1000000) {
          addConnector = false;
        }
      }
      if (addConnector) {
        if (!connectorNodeMap.has(tx.from)) connectorNodeMap.set(tx.from, new Set());
        connectorNodeMap.get(tx.from)!.add(toMain);
      }
    }
  });
});
// console.log(connectorNodeMap);
// --- Build Connection Map from Connector Node Map ---
// This map records direct links between two main accounts along with the external connector(s) that link them.
const connectionMap = new Map<string, string[]>();
connectorNodeMap.forEach((mainAccounts, externalId) => {
  // Only consider connectors that connect two or more main accounts.
  if (mainAccounts.size <= 1) return;

  // At this stage, we already handled spammer filtering per transaction.
  // (We still filter out defi here.)
  if (allMap.has(externalId)) {
    const acc = allMap.get(externalId)!;
    const extType = acc.ty.toLowerCase();
    if (extType === "defi") return;
    // Do not filter spammer here because the per-transaction check should have skipped low-value ones.
  }

  const mainList = Array.from(mainAccounts);
  // For every pair of main accounts, record that they are connected via this external account.
  for (let i = 0; i < mainList.length; i++) {
    for (let j = i + 1; j < mainList.length; j++) {
      // Create a stable key by sorting the two main account IDs.
      const key = [mainList[i], mainList[j]].sort().join("-");
      if (!connectionMap.has(key)) {
        connectionMap.set(key, [externalId]);
      } else {
        connectionMap.get(key)!.push(externalId);
      }
    }
  }
});

// --- Create Direct Links for Connector Connections ---
// Iterate over each pair of main accounts (from connectionMap) and create or update a direct link.
connectionMap.forEach((externalIds, key) => {
  // Filter out connectors that come from a spammer.
  const validExternalIds = externalIds.filter(externalId => {
    if (allMap.has(externalId)) {
      const acc = allMap.get(externalId)!;
      // Exclude if the connector account is flagged as spammer.
      return acc.ty.toLowerCase() !== "spammer";
    }
    return true;
  });

  if (validExternalIds.length === 0) return; // Skip creating a link if no valid connector exists.

  const [mainA, mainB] = key.split("-");

  // Optionally, skip links based on account types.
  const typeA = allMap.get(mainA)?.ty.toLowerCase();
  const typeB = allMap.get(mainB)?.ty.toLowerCase();
  if (typeA && typeB && ((typeA === "cex" && typeB === "cex") ||
      (typeA === "foundation" && typeB === "foundation"))) return;

  // If a direct link already exists between mainA and mainB, append the connector details.
  const existing = links.find(l =>
    (l.source === mainA && l.target === mainB) ||
    (l.source === mainB && l.target === mainA)
  );
  if (existing) {
    if (existing.connectors) {
      existing.connectors.push(...validExternalIds);
    } else {
      existing.connectors = validExternalIds;
    }
  } else {
    // Otherwise, create a new direct link with the connector details attached.
    links.push({
      source: mainA,
      target: mainB,
      direction: Direction.SEND, // Adjust based on your domain logic.
      connectors: validExternalIds
    });
  }
});
  const nodes = Array.from(nodeMap.values());

  return { nodes, links };
}

function initials(name: string): string {
  return name.replace(/\s+/g, "").slice(0, 2);
}
