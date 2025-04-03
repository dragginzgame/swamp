import { useEffect, useState } from "react";
import { AccountData, GraphNode } from "./types";
import { useWindowSize } from "./hooks/useWindowSize";
import { GraphContainer } from "./GraphContainer";
import { ToastMessage, useToast } from "./utils/Toast";


export default function Home() {

    const [loading, setLoading] = useState<boolean>(true);
    const [data, setData] = useState<AccountData[]>([]);
    useEffect(() => {
      async function loadData() {
        try {
          const categories = [
            "cex",
            "defi",
            "foundation",
            "identified",
            "nodeprovider",
            "spammer",
            "sns",
            "snsparticipant",
            "suspect"
          ];
          const fetchPromises = categories.map((cat) =>
            fetch(`/account_transactions_${cat}.json`).then((res) => res.json())
          );
          const results = await Promise.all(fetchPromises);
          const mergedData: AccountData[] = results.flat();
          setData(mergedData);
        } catch (err) {
          console.error("Failed to load JSON:", err);
        } finally {
          setLoading(false);
        }
      }
      loadData();
    }, []);
    const { setToastData } = useToast();

  const updateToast = (obj: ToastMessage) => {
    setToastData(obj);
  };
    const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
    const handleNodeClick = (node: GraphNode) => {
      console.log("Node clicked:", node);
      setSelectedNode(node);
      const message : string[] = [];
      
      if (node.mainAccounts && node.mainAccounts.length > 0) {
        message.push("Connected Accounts:");
        message.push(`${node.mainAccounts.join("    ")}`);
      }
      if (node.extra_info && node.extra_info.tx_count) {
        message.push("Node Info:");
        message.push(`Number of transactions: ${node.extra_info.tx_count}`);
        message.push(`Start date: ${node.extra_info.start_date}`);
        message.push(`End date: ${node.extra_info.end_date}`);
        message.push(`Average ICP amount: ${node.extra_info.average_amount}`);
      }
      updateToast({
        title: `${node.label} ${node.group}  ${node.id}`,
        message: message
      });
    };
   // number of txs, start date, end date, average icp amount
    // console.log(selectedNode);
    const { width, height } = useWindowSize();
  
    return (
      <>
        <main>
          <div className="position-relative  overflow-hidden  m-md-3 text-center bg-body-tertiary">
            {/* <div className="row align-items-center">
              <div className='col-3'>
                <div className="card">
                  <div className="card-header">
                    <h5 className="card-title">Selected Node</h5>
                  </div>
                  <div className="card-body">
                    {selectedNode ? (
                      <div>
                        <p>
                          <strong>Label:</strong> {selectedNode.label}
                        </p>
                        <p className="account-id">
                          <strong>Account ID:</strong> {selectedNode.id}
                        </p>
                        <p>
                          <strong>Type:</strong> {selectedNode.group}
                        </p>
                      </div>
                    ) : (
                      <p>No node selected.</p>
                    )}
                  </div>
                </div>
              </div>
              <div className='col-5'>
                {selectedNode?.mainAccounts && selectedNode.mainAccounts.length > 0 ? (
                  <div>
                    <h6>List of connected accounts:</h6>
                    <ul className="list-group">
                      {selectedNode.mainAccounts.map((account) => (
                        <li key={account} className="list-group-item">
                          {account}
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : null}
              </div>
             <div className='col-4 h-25 overflow-auto'>
                {selectedNode?.defiTxs && selectedNode.defiTxs.length > 0 ? (
                  <div>
                    <h6>List of defi transfers:</h6>
                    <ul className="list-group">
                      {selectedNode.defiTxs.map((account) => (
                        <li key={account.to} className="list-group-item">
                          {account.to}
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : null}
              </div> 
            </div> */}
          </div>
          <div className="position-relative overflow-hidden  m-md-3 text-center bg-body-tertiary">
            <div className="container-fluid" style={{ height: "100vh" }}>
              <div className="row h-100">
                <div className="col-12 h-100 border">
                  <GraphContainer
                    data={data}
                    width={width} // adjust as needed
                    height={height}
                    onNodeClick={handleNodeClick}
                    loading={loading}
                  />
                </div>
              </div>
            </div>
  
          </div>
  
  
        </main>
  
        <footer className="container py-5">
          <div className="row">
            <div className="col-12 col-md">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                fill="none"
                stroke="currentColor"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                className="d-block mb-2"
                role="img"
                viewBox="0 0 24 24"
              >
                <title>Product</title>
                <circle cx="12" cy="12" r="10" />
                <path d="M14.31 8l5.74 9.94M9.69 8h11.48M7.38 12l5.74-9.94M9.69 16L3.95 6.06M14.31 16H2.83m13.79-4l-5.74 9.94" />
              </svg>
              <small className="d-block mb-3 text-body-secondary">&copy; 2025</small>
            </div>
          </div>
        </footer>
      </>
    )
  }
  