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

const excludedTypes = ["defi", "spammer"];

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
  const links: GraphLink[] = [];
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
  const hiddenLinksMap = new Map<string, GraphLink[]>();
 // --- Link creation between known nodes ---
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

  // ✳️ If either node is defi, store it in the hidden map
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
const connectorNodeMap = new Map<string, Set<string>>();
allMap.forEach((acc: AccountData) => {
  acc.transactions.forEach(tx => {
    if (tx.op_type !== "Transfer") return;

    const fromMain = accountToMain.get(tx.from);
    const toMain = accountToMain.get(tx.to);

    if (fromMain && toMain && fromMain === toMain) return; // skip internal

    const fromIsKnown = fromMain && nodeMap.has(fromMain);
    const toIsKnown = toMain && nodeMap.has(toMain);

    if (fromIsKnown && !toMain) {
      if (!connectorNodeMap.has(tx.to)) {
        connectorNodeMap.set(tx.to, new Set());
      }
      connectorNodeMap.get(tx.to)!.add(fromMain);
    }

    if (toIsKnown && !fromMain) {
      if (!connectorNodeMap.has(tx.from)) {
        connectorNodeMap.set(tx.from, new Set());
      }
      connectorNodeMap.get(tx.from)!.add(toMain);
    }
  });
});

// --- Create Connector Nodes ---
connectorNodeMap.forEach((mainAccounts, externalId) => {
  if (mainAccounts.size <= 1) return;


  if (allMap.has(externalId)) {
    const acc = allMap.get(externalId)!;
    const ty = acc.ty.toLowerCase();
    if (ty === "defi") return;
  }

  const mainList = Array.from(mainAccounts);

  // 🛑 Check if any two main nodes already have a direct link — if so, skip connector node
  const hasDirectLink = mainList.some((a, idx) =>
    mainList.slice(idx + 1).some(b =>
      links.some(link =>
        (link.source === a && link.target === b) || (link.source === b && link.target === a)
      )
    )
  );

  if (hasDirectLink) return;

  const types = mainList.map(id => allMap.get(id)?.ty.toLowerCase());
  if (types.every(t => t === "cex") || types.every(t => t === "foundation")) return;

  const label = Array.from(mainAccounts)
    .map(id => allMap.get(id)?.name || "")
    .map(name => name.split(" ").map(w => w[0]).join(""))
    .join("/");

  nodeMap.set(externalId, {
    id: externalId,
    label,
    group: "connector"
  });

  mainAccounts.forEach(mainId => {
    links.push({
      source: mainId,
      target: externalId,
      direction: Direction.SEND
    });
  });
});

  // const hiddenTxMap = new Map<string, any[]>();
  // data.forEach(acc => {
  //   acc.transactions.forEach(tx => {
  //     if (tx.op_type === "Transfer") {
  //       const from = tx.from;
  //       const to = tx.to;
  //       const fromData = allMap.get(from);
  //       const toData = allMap.get(to);
  //       if (
  //         (fromData && fromData.ty.toLowerCase() === "defi") ||
  //         (toData && toData.ty.toLowerCase() === "defi")
  //       ) {
  //         if (mainMap.has(from)) {
  //           if (!hiddenTxMap.has(from)) {
  //             hiddenTxMap.set(from, []);
  //           }
  //           hiddenTxMap.get(from)!.push(tx);
  //         }
  //         if (mainMap.has(to)) {
  //           if (!hiddenTxMap.has(to)) {
  //             hiddenTxMap.set(to, []);
  //           }
  //           hiddenTxMap.get(to)!.push(tx);
  //         }
  //       }
  //     }
  //   });
  // });

 
  // nodeMap.forEach((node, id) => {
  //   node.defiTxs = hiddenTxMap.get(id) || [];
  // });
  // console.log(nodeMap);
  const nodes = Array.from(nodeMap.values());

  return { nodes, links };
}

function initials(name: string): string {
  return name.replace(/\s+/g, "").slice(0, 2);
}
