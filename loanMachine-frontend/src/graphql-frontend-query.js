

const VITE_SUBGRAPH_URL = import.meta.env.VITE_SUBGRAPH_URL;
export async function fetchDonations() {
  const query = `
    {
      donations(first: 20) {
        id
        donor { id }
        amount
        totalDonation
      }
    }
  `;
  const response = await fetch(VITE_SUBGRAPH_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });
  const { data } = await response.json();
  return data?.donations || [];
}


export async function fetchContractStats() {
  const query = `
    query GetStats {
      stats(id: "singleton") {
        id
        totalDonations
        totalBorrowed
        availableBalance
      }
    }
  `;

  try {
    const response = await fetch(VITE_SUBGRAPH_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query }),
    });
    
    const { data } = await response.json();
    
    if (data && data.stats) {
      return {
        totalDonations: data.stats.totalDonations,
        totalBorrowed: data.stats.totalBorrowed,
        availableBalance: data.stats.availableBalance,
        contractBalance: data.stats.availableBalance // Using availableBalance as contract balance
      };
    }
    
    return {
      totalDonations: "0",
      totalBorrowed: "0",
      availableBalance: "0",
      contractBalance: "0"
    };
  } catch (error) {
    console.error("Error fetching contract stats:", error);
    throw error;
  }
}

export async function fetchLastTransactions({ limit = 5 } = {}) {
  const query = `
    query GetLastTransactions {
      donations(first: ${limit}, orderBy: timestamp, orderDirection: desc) {
        id
        donor {
          id
        }
        amount
        timestamp
      }
      borrows(first: ${limit}, orderBy: timestamp, orderDirection: desc) {
        id
        borrower {
          id
        }
        amount
        timestamp
      }
      repayments(first: ${limit}, orderBy: timestamp, orderDirection: desc) {
        id
        borrower {
          id
        }
        amount
        timestamp
      }
    }
  `;

  try {
    const response = await fetch(VITE_SUBGRAPH_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query }),
    });
    
    const { data } = await response.json();
    
    // Combine and sort all transactions by timestamp
    const allTransactions = [
      ...(data.donations || []).map(tx => ({ ...tx, type: "donation" })),
      ...(data.borrows || []).map(tx => ({ ...tx, type: "borrow" })),
      ...(data.repayments || []).map(tx => ({ ...tx, type: "repayment" }))
    ].sort((a, b) => b.timestamp - a.timestamp).slice(0, limit);
    
    return allTransactions;
  } catch (error) {
    console.error("Error fetching last transactions:", error);
    throw error;
  }
}