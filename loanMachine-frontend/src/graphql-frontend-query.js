

const SUBGRAPH_URL = import.meta.env.REACT_APP_SUBGRAPH_URL;

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
  const response = await fetch(SUBGRAPH_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });
  const { data } = await response.json();
  return data?.donations || [];
}
