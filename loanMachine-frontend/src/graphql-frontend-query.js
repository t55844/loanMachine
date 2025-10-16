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
export async function fetchLastTransactions({ limit = 5 } = {}) {
  const query = `
    query GetLastTransactions($limit: Int!) {
      donatedEvents(first: $limit, orderBy: blockTimestamp, orderDirection: desc) {
        id
        donor
        amount
        blockTimestamp
      }
      borrowedEvents(first: $limit, orderBy: blockTimestamp, orderDirection: desc) {
        id
        borrower
        amount
        blockTimestamp
      }
      repaidEvents(first: $limit, orderBy: blockTimestamp, orderDirection: desc) {
        id
        borrower
        amount
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query, { limit });

    const allTransactions = [
      ...(data?.donatedEvents || []).map((tx) => ({ ...tx, type: "donation", timestamp: tx.blockTimestamp })),
      ...(data?.borrowedEvents || []).map((tx) => ({ ...tx, type: "borrow", timestamp: tx.blockTimestamp, borrower: { id: tx.borrower } })),
      ...(data?.repaidEvents || []).map((tx) => ({ ...tx, type: "repayment", timestamp: tx.blockTimestamp, borrower: { id: tx.borrower } })),
    ]
      .sort((a, b) => Number(b.timestamp) - Number(a.timestamp))
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
      loanRequisitionCreatedEvents(orderBy: blockTimestamp, orderDirection: desc) {
        id
        requisitionId
        borrower
        amount
        parcelsCount
        blockTimestamp
      }
    }
  `;

  try {
    const data = await client.request(query);
    return (data.loanRequisitionCreatedEvents || []).map(event => ({
      id: event.id,
      requisitionId: event.requisitionId,
      borrower: event.borrower,
      amount: event.amount,
      currentCoveragePercentage: 0, // Default, as no coverage tracking in events
      minimumCoverage: 0, // Default
      status: "Pending", // Assume pending
      durationDays: 0, // Default
      creationTime: event.blockTimestamp,
      coveringLendersCount: 0, // Default
      parcelsCount: event.parcelsCount
    }));
  } catch (error) {
    console.error("Error fetching loan requisitions:", error);
    return [];
  }
}

/* -----------------------------------------------------------
   Fetch User Requisitions
----------------------------------------------------------- */
export async function fetchUserRequisitions(userAddress) {
  const query = `
    query GetUserRequisitions($userAddress: Bytes!) {
      loanRequisitionCreatedEvents(where: { borrower: $userAddress }, orderBy: blockTimestamp, orderDirection: desc) {
        id
        requisitionId
        borrower
        amount
        parcelsCount
        blockTimestamp
        # Note: funded and fundedAt would require cross-referencing with LoanFundedEvent
      }
    }
  `;

  try {
    const data = await client.request(query, {
      userAddress: userAddress.toLowerCase(),
    });
    return (data.loanRequisitionCreatedEvents || []).map(event => ({
      id: event.id,
      requisitionId: event.requisitionId,
      borrower: event.borrower,
      amount: event.amount,
      currentCoveragePercentage: 0, // Default
      minimumCoverage: 0, // Default
      status: "Pending", // Assume
      durationDays: 0, // Default
      creationTime: event.blockTimestamp,
      coveringLendersCount: 0, // Default
      parcelsCount: event.parcelsCount,
      funded: false, // Default
      fundedAt: null // Default
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
      donatedEvents(where: { donor: $userAddress }) {
        id
        amount
        blockTimestamp
      }
      borrowedEvents(where: { borrower: $userAddress }) {
        id
        amount
        blockTimestamp
      }
      repaidEvents(where: { borrower: $userAddress }) {
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

    const totalDonated = donations.reduce((sum, d) => sum + Number(d.amount), 0).toString();
    const totalBorrowed = borrows.reduce((sum, b) => sum + Number(b.amount), 0).toString();
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
  const query = `
    query GetWalletMember($wallet: Bytes!) {
      memberToWalletVinculationEvents(where: { wallet: $wallet }, first: 1, orderBy: blockTimestamp, orderDirection: desc) {
        memberId
      }
    }
  `;

  try {
    const data = await client.request(query, {
      wallet: walletAddress.toLowerCase(),
    });

    const event = data.memberToWalletVinculationEvents?.[0];
    if (!event) return null;

    return {
      memberId: event.memberId,
      wallets: [walletAddress.toLowerCase()],
      currentReputation: 0 // Default, as no reputation tracking in events
    };
  } catch (error) {
    console.error("Error fetching wallet member:", error);
    throw error;
  }
}



export async function fetchCompleteMemberData(walletAddress, memberId = null) {
  // If memberId is not provided, first get it from the wallet
  let targetMemberId = memberId;
  
  if (!targetMemberId) {
    const memberQuery = `
      query GetWalletMember($wallet: Bytes!) {
        walletVinculateds(where: { wallet: $wallet }, first: 1) {
          memberId
          vinculationEvent {
            blockTimestamp
          }
        }
      }
    `;

    try {
      const memberData = await client.request(memberQuery, {
        wallet: walletAddress.toLowerCase(),
      });

      const walletVinculated = memberData.walletVinculateds?.[0];
      if (!walletVinculated) return null;
      
      targetMemberId = walletVinculated.memberId;
    } catch (error) {
      console.error("Error fetching wallet member:", error);
      throw error;
    }
  }

  // Now fetch complete member data including reputation and all wallets
  const completeQuery = `
    query GetCompleteMemberData($wallet: Bytes!, $memberId: Int!) {
      # Get the specific wallet details
      specificWallet: walletVinculateds(where: { wallet: $wallet }, first: 1) {
        id
        memberId
        wallet
        walletInfo
        vinculationEvent {
          id
          timestamp
          blockTimestamp
          transactionHash
        }
      }
      
      # Get all wallets for this member
      allMemberWallets: walletVinculateds(where: { memberId: $memberId }) {
        id
        wallet
        walletInfo
        vinculationEvent {
          timestamp
          blockTimestamp
        }
      }
      
      # Get latest reputation for this member
      reputation: reputationChangedEvents(
        where: { memberId: $memberId }
        orderBy: timestamp
        orderDirection: desc
        first: 1
      ) {
        memberId
        newReputation
        points
        increase
        timestamp
      }
      
      # Get member's vote history count
      voteHistory: voteCastEvents(where: { memberId: $memberId }) {
        id
        electionId
        candidateId
        voteWeight
      }
      
      # Get loan requisitions by this wallet
      loanRequisitions: loanRequisitionCreatedEvents(where: { borrower: $wallet }) {
        id
        requisitionId
        amount
        parcelsCount
        blockTimestamp
      }
      
      # Get donation history for this wallet
      donations: donatedEvents(where: { donor: $wallet }) {
        id
        amount
        totalDonation
        blockTimestamp
      }
    }
  `;

  try {
    const completeData = await client.request(completeQuery, {
      wallet: walletAddress.toLowerCase(),
      memberId: targetMemberId
    });

    // Process and merge the data
    const specificWallet = completeData.specificWallet?.[0];
    const allWallets = completeData.allMemberWallets || [];
    const latestReputation = completeData.reputation?.[0];
    const voteHistory = completeData.voteHistory || [];
    const loanRequisitions = completeData.loanRequisitions || [];
    const donations = completeData.donations || [];

    if (!specificWallet) return null;

    return {
      memberId: targetMemberId,
      wallets: allWallets.map(w => w.wallet),
      currentReputation: latestReputation?.newReputation || 0,
      reputationHistory: completeData.reputation,
      voteCount: voteHistory.length,
      totalVoteWeight: voteHistory.reduce((sum, vote) => sum + vote.voteWeight, 0),
      loanRequisitionCount: loanRequisitions.length,
      totalLoansRequested: loanRequisitions.reduce((sum, loan) => sum + parseInt(loan.amount), 0),
      donationCount: donations.length,
      totalDonated: donations.reduce((sum, donation) => sum + parseInt(donation.amount), 0),
      walletDetails: allWallets,
      isModerator: false,
      lastUpdated: new Date().toISOString()
    };
  } catch (error) {
    console.error("Error fetching complete member data:", error);
    throw error;
  }
}
