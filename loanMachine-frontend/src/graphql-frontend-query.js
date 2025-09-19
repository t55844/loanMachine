

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


export async function GetAllData() {
  // All donations
  const query = `donations(orderBy: timestamp, orderDirection: desc) {
    id
    donor {
      id
    }
    amount
    timestamp
    totalDonation
  }

  borrows(orderBy: timestamp, orderDirection: desc) {
    id
    borrower {
      id
    }
    amount
    timestamp
    totalBorrowing
  }

  repayments(orderBy: timestamp, orderDirection: desc) {
    id
    borrower {
      id
    }
    amount
    timestamp
    remainingDebt
  }`

const response = await fetch(VITE_SUBGRAPH_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });
  const { data } = await response.json();
  return data?.donations || [];

}