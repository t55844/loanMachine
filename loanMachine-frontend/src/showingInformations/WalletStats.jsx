import { useState } from "react";
import { ethers } from "ethers";

const RPC_URL = import.meta.env.VITE_RPC_URL;
const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;

const ABI = [
  "function getUserStats(address _user) view returns (uint256 userDonations, uint256 userBorrowings, uint256 lastBorrow, bool canBorrowNow)",
  "function getDonation(address _user) view returns (uint256)",
  "function getBorrowing(address _user) view returns (uint256)",
  "function getLastBorrowTime(address _user) view returns (uint256)"
];

export default function WalletStats() {
  const [wallet, setWallet] = useState("");
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function fetchData() {
    if (!ethers.utils.isAddress(wallet)) {
      setError("Endereço inválido");
      return;
    }

    setLoading(true);
    setError("");
    setData(null);

    try {
      const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
      const contract = new ethers.Contract(CONTRACT_ADDRESS, ABI, provider);

      const [donations, borrowings, lastBorrow, canBorrowNow] =
        await contract.getUserStats(wallet);

      setData({
        donations: ethers.utils.formatEther(donations),
        borrowings: ethers.utils.formatEther(borrowings),
        lastBorrow: new Date(lastBorrow.toNumber() * 1000).toLocaleString(),
        canBorrowNow
      });
    } catch (e) {
      console.error("Erro ao buscar dados:", e);
      setError("Não foi possível carregar os dados para essa carteira");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="stats-box">
      <h2>User Stats</h2>

      <div className="wallet-input-row">
        <input
          type="text"
          placeholder="Insira o endereço da carteira"
          value={wallet}
          onChange={(e) => setWallet(e.target.value)}
          className="wallet-input"
        />
        <button onClick={fetchData} className="wallet-button">
          Buscar
        </button>
      </div>

      {loading && <p>Carregando dados...</p>}
      {error && <p className="error">{error}</p>}

      {data && (
        <div className="stats-grid">
          <div><strong>Donations:</strong> {data.donations} ETH</div>
          <div><strong>Borrowings:</strong> {data.borrowings} ETH</div>
          <div><strong>Last Borrow:</strong> {data.lastBorrow}</div>
          <div><strong>Can Borrow Now:</strong> {data.canBorrowNow ? "Yes" : "No"}</div>
        </div>
      )}
    </div>
  );
}
