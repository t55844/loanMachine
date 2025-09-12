

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
