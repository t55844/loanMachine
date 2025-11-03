import { request, GraphQLClient } from "graphql-request";

const GRAPHQL_URL = import.meta.env.VITE_SUBGRAPH_URL;

// Optional: use a single client instance for all queries
const client = new GraphQLClient(GRAPHQL_URL, {
  headers: { "Content-Type": "application/json" },
});

/* -----------------------------------------------------------
   Fetch Donations and Borrows
----------------------------------------------------------- */
export async function fetchDonationsAndBorrows() {
  const query = `
    {
      donatedEvents(first: 10, orderBy: blockTimestamp, orderDirection: desc) {
        id
        donor
        amount
        totalDonation
        blockTimestamp
      }
      borrowedEvents(first: 10, orderBy: blockTimestamp, orderDirection: desc) {
        id
        borrower
        amount
        totalBorrowing
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query);
    const donations = (data?.donatedEvents || []).map(event => ({
      id: event.id,
      donor: { id: event.donor },
      amount: event.amount,
      totalDonation: event.totalDonation,
      timestamp: event.blockTimestamp
    }));
    const borrows = (data?.borrowedEvents || []).map(event => ({
      id: event.id,
      borrower: { id: event.borrower },
      amount: event.amount,
      totalBorrowing: event.totalBorrowing,
      timestamp: event.blockTimestamp
    }));
    return [donations, borrows];
  } catch (error) {
    console.error("Error fetching donations and borrows:", error);
    return [[], []];
  }
}

/* -----------------------------------------------------------
   Fetch Contract Stats
----------------------------------------------------------- */
export async function fetchContractStats() {
  const query = `
    query GetStats {
      totalDonationsUpdatedEvents(first: 1, orderBy: blockTimestamp, orderDirection: desc) {
        total
      }
      totalBorrowedUpdatedEvents(first: 1, orderBy: blockTimestamp, orderDirection: desc) {
        total
      }
      availableBalanceUpdatedEvents(first: 1, orderBy: blockTimestamp, orderDirection: desc) {
        total
      }
    }
  `;

  try {
    const data = await client.request(query);
    const totalDonations = data?.totalDonationsUpdatedEvents?.[0]?.total || "0";
    const totalBorrowed = data?.totalBorrowedUpdatedEvents?.[0]?.total || "0";
    const availableBalance = data?.availableBalanceUpdatedEvents?.[0]?.total || "0";

    return {
      totalDonations,
      totalBorrowed,
      availableBalance,
      contractBalance: availableBalance,
    };
  } catch (error) {
    console.error("Error fetching contract stats:", error);
    throw error;
  }
}

/* -----------------------------------------------------------
   Fetch Last Transactions
----------------------------------------------------------- */
const parseCustomDate = (dateString) => {
  if (!dateString) return new Date(0); // Return epoch start for safety
  
  // Example: "23/10/2025 22:33:31"
  const [datePart, timePart] = dateString.split(' ');
  const [day, month, year] = datePart.split('/');
  
  // Create a standard ISO-like string format: YYYY-MM-DDTHH:mm:ss for reliable Date parsing.
  const isoString = `${year}-${month}-${day}T${timePart}`;
  return new Date(isoString);
};


export async function fetchLastTransactions({ limit = 5 } = {}) {
  // Use a much larger limit for the initial query (e.g., 10 times the requested limit) 
  // to ensure we capture the true 'limit' number of most recent transactions across all types.
  const fetchLimit = limit * 10; 

  const query = `
    query GetLastTransactions($fetchLimit: Int!) {
      donatedEvents(first: $fetchLimit, orderBy: blockTimestamp, orderDirection: desc) {
        id
        donor
        amount
        blockTimestamp
      }
      borrowedEvents(first: $fetchLimit, orderBy: blockTimestamp, orderDirection: desc) {
        id
        borrower
        amount
        blockTimestamp
      }
      repaidEvents(first: $fetchLimit, orderBy: blockTimestamp, orderDirection: desc) {
        id
        borrower
        amount
        blockTimestamp
      }
      withdrawnEvents(first: $fetchLimit, orderBy: blockTimestamp, orderDirection: desc) {
        id
        donor
        amount
        blockTimestamp
      }
    }
  `;

  try {
    // Pass the larger fetchLimit to the GraphQL client
    const data = await client.request(query, { fetchLimit });

    const allTransactions = [
      ...(data?.donatedEvents || []).map((tx) => ({ ...tx, type: "donation", timestamp: tx.blockTimestamp })),
      ...(data?.borrowedEvents || []).map((tx) => ({ ...tx, type: "borrow", timestamp: tx.blockTimestamp, borrower: { id: tx.borrower } })),
      ...(data?.repaidEvents || []).map((tx) => ({ ...tx, type: "repayment", timestamp: tx.blockTimestamp, borrower: { id: tx.borrower } })),
      ...(data?.withdrawnEvents || []).map((tx) => ({ ...tx, type: "withdrawn", timestamp: tx.blockTimestamp, donor: { id: tx.donor } })),
    ]
      // FIX: Use the custom parsing function to correctly sort the 'DD/MM/YYYY HH:mm:ss' strings chronologically.
      .sort((a, b) => {
        const dateA = parseCustomDate(a.timestamp);
        const dateB = parseCustomDate(b.timestamp);
        
        // Sort descending (most recent first): dateB - dateA
        return dateB.getTime() - dateA.getTime();
      })
      // Slice down to the originally requested limit (e.g., 5)
      .slice(0, limit);

    return allTransactions;
  } catch (error) {
    console.error("Error fetching last transactions:", error);
    throw error;
  }
}

/* -----------------------------------------------------------
   Fetch Pending Loan Requisitions
----------------------------------------------------------- */
export async function fetchLoanRequisitions() {
  const query = `
    query GetPendingRequisitions {
      loanRequisitionCreatedCancelledEvents(
        where: {
          status_in: [0, 1, 2, 3]  # Pending, PartiallyCovered, FullyCovered, Active
        }
        orderBy: blockTimestamp
        orderDirection: desc
      ) {
        id
        requisitionId
        borrower
        amount
        parcelsCount
        status
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query);
    return (data.loanRequisitionCreatedCancelledEvents || []).map(event => ({
      id: event.id,
      requisitionId: event.requisitionId,
      borrower: event.borrower,
      amount: event.amount,
      currentCoveragePercentage: 0,
      minimumCoverage: 0,
      status: getStatusText(event.status), // Helper function to convert status number to text
      durationDays: 0,
      creationTime: event.blockTimestamp,
      coveringLendersCount: 0,
      parcelsCount: event.parcelsCount
    }));
  } catch (error) {
    console.error("Error fetching loan requisitions:", error);
    return [];
  }
}

// Helper function to convert status number to text
function getStatusText(status) {
  const statusMap = {
    0: "Pending",
    1: "PartiallyCovered", 
    2: "FullyCovered",
    3: "Active",
    4: "Repaid",
    5: "Defaulted",
    6: "Cancelled"
  };
  return statusMap[status] || "Unknown";
}

/* -----------------------------------------------------------
   Fetch User Requisitions
----------------------------------------------------------- */
export async function fetchUserRequisitions(userAddress) {
  const query = `
    query GetUserRequisitions($userAddress: Bytes!) {
      loanRequisitionCreatedCancelledEvents(
        where: { 
          borrower: $userAddress,
          status_not_in: [6] // Exclude cancelled requisitions
        }
        orderBy: blockTimestamp
        orderDirection: desc
      ) {
        id
        requisitionId
        borrower
        amount
        parcelsCount
        status
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query, {
      userAddress: userAddress.toLowerCase(),
    });
    return (data.loanRequisitionCreatedCancelledEvents || []).map(event => ({
      id: event.id,
      requisitionId: event.requisitionId,
      borrower: event.borrower,
      amount: event.amount,
      currentCoveragePercentage: 0,
      minimumCoverage: 0,
      status: getStatusText(event.status),
      durationDays: 0,
      creationTime: event.blockTimestamp,
      coveringLendersCount: 0,
      parcelsCount: event.parcelsCount,
      funded: event.status >= 2, // FullyCovered or Active status
      fundedAt: event.status >= 2 ? event.blockTimestamp : null
    }));
  } catch (error) {
    console.error("Error fetching user requisitions:", error);
    return [];
  }
}

/* -----------------------------------------------------------
   Fetch User Donations
----------------------------------------------------------- */
export async function fetchUserDonations(userAddress) {
  const query = `
    query GetUserDonations($userAddress: Bytes!) {
      donatedEvents(where: { donor: $userAddress }, orderBy: blockTimestamp, orderDirection: desc) {
        id
        amount
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query, {
      userAddress: userAddress.toLowerCase(),
    });
    return data.donatedEvents || [];
  } catch (error) {
    console.error("Error fetching user donations:", error);
    return [];
  }
}

/* -----------------------------------------------------------
   Fetch User Data
----------------------------------------------------------- */
export async function fetchUserData(userAddress) {
  const query = `
    query GetUserData($userAddress: Bytes!) {
      donatedEvents(where: { donor: $userAddress },orderDirection: desc, orderBy: blockTimestamp, first: 1) {
        id
        totalDonation
        blockTimestamp
      }
      borrowedEvents(where: { borrower: $userAddress },orderDirection: desc, orderBy: blockTimestamp, first: 1) {
        id
        totalBorrowing
        blockTimestamp
      }
      repaidEvents(where: { borrower: $userAddress },orderDirection: desc, orderBy: blockTimestamp, first: 1) {
        id
        amount
        blockTimestamp
        remainingDebt
      }
    }
  `;

  try {
    const data = await client.request(query, {
      userAddress: userAddress.toLowerCase(),
    });

    const donations = data.donatedEvents || [];
    const borrows = data.borrowedEvents || [];
    const repayments = data.repaidEvents || [];

    const totalDonated = donations.reduce((sum, d) => sum + Number(d.totalDonation), 0).toString();
    const totalBorrowed = borrows.reduce((sum, b) => sum + Number(b.totalBorrowing), 0).toString();
    const currentDebt = repayments.length > 0 ? repayments[repayments.length - 1].remainingDebt : totalBorrowed;
    const lastActivity = Math.max(
      ...[
        ...donations.map(d => Number(d.blockTimestamp)),
        ...borrows.map(b => Number(b.blockTimestamp)),
        ...repayments.map(r => Number(r.blockTimestamp))
      ],
      0
    ).toString();

    return {
      id: userAddress,
      borrows: borrows.map(b => ({ id: b.id, amount: b.amount, timestamp: b.blockTimestamp })),
      donations: donations.map(d => ({ id: d.id, amount: d.amount, timestamp: d.blockTimestamp })),
      repayments: repayments.map(r => ({ id: r.id, amount: r.amount, timestamp: r.blockTimestamp })),
      currentDebt,
      lastActivity,
      totalDonated,
      totalBorrowed,
    };
  } catch (error) {
    console.error("Error fetching user data:", error);
    throw error;
  }
}

/* -----------------------------------------------------------
   Fetch Wallet Member
----------------------------------------------------------- */
export async function fetchWalletMember(walletAddress) {
  const query = `query GetWalletMember($wallet: Bytes!)
  {memberToWalletVinculationEvents(
    where: { wallet: $wallet }, first: 1, orderBy: blockTimestamp, orderDirection: desc){memberId}}`;

  try {
    const data = await client.request(query, {
      wallet: walletAddress.toLowerCase(),
    });

    const event = data.memberToWalletVinculationEvents?.[0];

    if (!event) {
      // 🛑 PATH 1: NO EVENT FOUND
      return { 
        hasVinculation: false, // Explicitly false
        memberId: null, 
        wallets: [],
      };
    }

    // ✅ PATH 2: EVENT FOUND (SUCCESS)
    return {
      memberId: event.memberId,
      wallets: [walletAddress.toLowerCase()],
      currentReputation: 0,
      hasVinculation: true, // 🚨 THIS IS THE MISSING KEY
    };
  } catch (error) {
    console.error("Error fetching wallet member:", error);
    throw error; // Allow Web3Context to catch this as a network error
  }
}


export async function fetchMemberReputation(memberId) {
  const query = `
    query GetMemberReputation($memberId: Int!) {
      reputationChangedEvents(
        where: { memberId: $memberId }
        orderBy: timestamp
        orderDirection: desc
        first: 1
      ) {
        newReputation
        points
        increase
        timestamp
      }
    }
  `;

  const variables = { memberId };
  
  try {
    const data = await request(GRAPHQL_URL, query, variables);
    
    if (data.reputationChangedEvents && data.reputationChangedEvents.length > 0) {
      return data.reputationChangedEvents[0].newReputation;
    }
    
    return 0; // Default reputation if no events found
  } catch (error) {
    console.error('Error fetching member reputation:', error);
    return 0; // Return 0 on error
  }
}


export async function fetchCompleteMemberData(walletAddress, memberId = null) {
  // If memberId is not provided, find it from MemberToWalletVinculationEvent
  let targetMemberId = memberId;
  
  if (!targetMemberId) {
    const memberQuery = `
      query GetWalletMember($wallet: Bytes!) {
        memberToWalletVinculationEvents(
          where: { wallet: $wallet }
          orderBy: blockTimestamp
          orderDirection: desc
          first: 1
        ) {
          memberId
          walletVinculated
          blockTimestamp
        }
      }
    `;

    try {
      const memberData = await client.request(memberQuery, {
        wallet: walletAddress.toLowerCase(),
      });

      const memberEvent = memberData.memberToWalletVinculationEvents?.[0];
      if (!memberEvent) return null;
      
      targetMemberId = memberEvent.memberId;
    } catch (error) {
      console.error("Error fetching wallet member:", error);
      throw error;
    }
  }

  const completeQuery = `
    query GetCompleteMemberData($memberId: Int!, $wallet: Bytes!) {
      # Get all wallet vinculations for this member
      memberWalletEvents: memberToWalletVinculationEvents(
        where: { memberId: $memberId }
        orderBy: blockTimestamp
        orderDirection: desc
      ) {
        wallet
        walletVinculated
        blockTimestamp
      }
      
      # Get latest reputation
      reputation: reputationChangedEvents(
        where: { memberId: $memberId }
        orderBy: timestamp
        orderDirection: desc
        first: 1
      ) {
        newReputation
        points
        increase
        timestamp
      }
      
      # Get vote count
      voteCount: voteCastEvents(where: { memberId: $memberId }) {
        id
      }
      
      # Get loan requisitions -
      loanRequisitions: loanRequisitionCreatedCancelledEvents(
        where: { 
          borrower: $wallet,
          status_not_in: [6]
        }
      ) {
        requisitionId
        amount
        status
        blockTimestamp
      }
      
      # Get donations
      donations: donatedEvents(where: { donor: $wallet }) {
        amount
      }
    }
  `;

  try {
    const data = await client.request(completeQuery, {
      wallet: walletAddress.toLowerCase(),
      memberId: targetMemberId
    });

    const latestWalletEvent = data.memberWalletEvents?.[0];
    if (!latestWalletEvent) return null;

    return {
      memberId: targetMemberId,
      wallets: latestWalletEvent.walletVinculated || [],
      currentReputation: data.reputation?.[0]?.newReputation || 0,
      voteCount: data.voteCount?.length || 0,
      loanRequisitionCount: data.loanRequisitions?.length || 0,
      totalLoansRequested: data.loanRequisitions?.reduce((sum, loan) => sum + parseInt(loan.amount), 0) || 0,
      donationCount: data.donations?.length || 0,
      totalDonated: data.donations?.reduce((sum, donation) => sum + parseInt(donation.amount), 0) || 0,
      lastActivity: latestWalletEvent.blockTimestamp,
      lastUpdated: new Date().toISOString(),
      // Include the actual requisitions for debugging
      requisitions: data.loanRequisitions || []
    };
  } catch (error) {
    console.error("Error fetching complete member data:", error);
    throw error;
  }
}


export async function fetchLastElection() {
  const query = `
    query LastElection {
      electionClosedEvents(orderDirection: desc, orderBy: blockTimestamp, first: 1) {
        electionId
        winnerId
        winningVotes
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query);
    const lastElection = data.electionClosedEvents?.[0];
    
    if (!lastElection) return null;

    return {
      electionId: lastElection.electionId,
      winnerId: lastElection.winnerId,
      winningVotes: lastElection.winningVotes,
      blockTimestamp: lastElection.blockTimestamp
    };
  } catch (error) {
    console.error("Error fetching last election:", error);
    throw error;
  }
}

export async function fetchBorrowerRequisitions(borrower) {
  const query = `
    query GetBorrowerRequisitions($wallet: String!) {
      loanRequisitions: loanRequisitionCreatedCancelledEvents(
        where: { 
          borrower: $wallet,
          status_not_in: [6]
        }
      ) {
        requisitionId
        amount
        status
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query, { wallet: borrower.toLowerCase() });
    const requisitions = data.loanRequisitions || [];
    
    return requisitions.map(req => ({
      id: req.requisitionId,
      amount: (parseFloat(req.amount) / 1e6).toFixed(2),
      status: Number(req.status),
      creationTime: new Date(Number(req.blockTimestamp) * 1000).toLocaleString(),
      // Set default values for fields not in the query
      minimumCoverage: 0,
      currentCoverage: 0,
      coveringLendersCount: 0
    }));
  } catch (error) {
    console.error("Error fetching borrower requisitions:", error);
    throw error;
  }
}

