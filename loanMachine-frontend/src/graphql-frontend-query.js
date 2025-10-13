

const VITE_SUBGRAPH_URL = import.meta.env.VITE_SUBGRAPH_URL;
export async function fetchDonationsAndBorrows() {
  const query = `
    {
      donations(first: 10) {
        id
        donor { id }
        amount
        totalDonation
      }
      borrows(first: 10) {
        id
        borrower { id }
        amount
        totalBorrowing
      }
    }
  `;
  const response = await fetch(VITE_SUBGRAPH_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });
  const { data } = await response.json(); // Changed from allData to data
  return [
    data?.donations || [],
    data?.borrows || []
  ];
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


export async function fetchLoanRequisitions() {
  const query = `
    query GetLoanRequisitions {
      loanRequests {
        requisitionId
        amount
        borrower {
          id
        }
      }
    }
  `;

  try {
    const response = await fetch(import.meta.env.VITE_SUBGRAPH_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query }),
    });
    
    const { data } = await response.json();
    return data.loanRequests || [];
  } catch (error) {
    console.error("Error fetching loan requisitions:", error);
    throw error;
  }
}

export async function fetchUserDonations(donorAddress) {
  const query = `
    query GetDonationsByDonor($donor: String!) {
      donations(where: { donor: $donor }) {
        id
        donor {
          id
        }
        amount
        totalDonation
      }
    }
  `;

  try {
    const response = await fetch(import.meta.env.VITE_SUBGRAPH_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ 
        query,
        variables: { donor: donorAddress.toLowerCase() }
      }),
    });
    
    const { data } = await response.json();
    return data.donations || [];
  } catch (error) {
    console.error("Error fetching user donations:", error);
    throw error;
  }
}


// GraphQL query functions
export async function fetchUserData(userAddress) {
  const query = `
    query GetUserData($userAddress: String!) {
      users(where: { id: $userAddress }) {
        id
        borrows {
          id
          amount
          timestamp
        }
        donations {
          id
          amount
          timestamp
        }
        repayments {
          id
          amount
          timestamp
        }
        currentDebt
        lastActivity
        totalDonated
        totalBorrowed
      }
    }
  `;

  try {
    const response = await fetch(import.meta.env.VITE_SUBGRAPH_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ 
        query,
        variables: { userAddress: userAddress.toLowerCase() }
      }),
    });
    
    const { data } = await response.json();
    
    if (data && data.users && data.users.length > 0) {
      return data.users[0];
    }

    // Return default data if user not found
    return {
      id: userAddress,
      borrows: [],
      donations: [],
      repayments: [],
      currentDebt: "0",
      lastActivity: "0",
      totalDonated: "0",
      totalBorrowed: "0"
    };
  } catch (error) {
    console.error("Error fetching user data:", error);
    throw error;
  }
}
export async function fetchWalletMember(walletAddress) {
  const query = `
    query GetWalletMember($wallet: String!) {
      members(where: {wallets_contains_nocase: [$wallet]}) {
        memberId
        wallets
        currentReputation
      }
    }
  `;

  try {
    const response = await fetch(import.meta.env.VITE_SUBGRAPH_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ 
        query,
        variables: {
          wallet: walletAddress.toLowerCase()
        }
      }),
    });
    
    const { data, errors } = await response.json();
    
    if (errors) {
      console.error("GraphQL errors:", errors);
      throw new Error(errors[0].message);
    }
    
    // Return the first member if exists, otherwise null
    return data.members && data.members.length > 0 ? data.members[0] : null;
  } catch (error) {
    console.error("Error fetching wallet member:", error);
    throw error;
  }
}