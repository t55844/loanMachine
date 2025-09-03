import { useEffect, useState } from "react";
import { ethers } from "ethers";
import { CONTRACT_ADDRESS, CONTRACT_ABI } from "./config";

function App() {
  const [account, setAccount] = useState("");
  const [contract, setContract] = useState(null);
  const [userStats, setUserStats] = useState({});
  const [contractBalance, setContractBalance] = useState("0");
  const [donationAmount, setDonationAmount] = useState("");
  const [borrowAmount, setBorrowAmount] = useState("");

  // Conectar com Metamask
  async function connectWallet() {
    if (window.ethereum) {
      const accounts = await window.ethereum.request({
        method: "eth_requestAccounts",
      });
      setAccount(accounts[0]);
      initContract();
    } else {
      alert("Instale o Metamask para usar esta aplicação!");
    }
  }

  // Inicializar contrato
  function initContract() {
    const provider = new ethers.providers.Web3Provider(window.ethereum);
    const signer = provider.getSigner();
    const loanContract = new ethers.Contract(
      CONTRACT_ADDRESS,
      CONTRACT_ABI,
      signer
    );
    setContract(loanContract);
  }

  // Buscar informações do contrato e usuário
  async function fetchData() {
    if (!contract || !account) return;
    const [donations, borrowings, lastBorrow, canBorrowNow] =
      await contract.getUserStats(account);
    setUserStats({
      donations: ethers.utils.formatEther(donations),
      borrowings: ethers.utils.formatEther(borrowings),
      lastBorrow: lastBorrow.toString(),
      canBorrowNow,
    });
    const balance = await contract.getContractBalance();
    setContractBalance(ethers.utils.formatEther(balance));
  }

  // Doar
  async function donate() {
    if (!contract || !donationAmount) return;
    const tx = await contract.donate({
      value: ethers.utils.parseEther(donationAmount),
    });
    await tx.wait();
    setDonationAmount("");
    fetchData();
  }

  // Pegar emprestado
  async function borrow() {
    if (!contract || !borrowAmount) return;
    const tx = await contract.borrow(ethers.utils.parseEther(borrowAmount));
    await tx.wait();
    setBorrowAmount("");
    fetchData();
  }

  // Reembolsar
  async function repay() {
    if (!contract || userStats.borrowings === "0") return;
    const tx = await contract.repay({
      value: ethers.utils.parseEther(userStats.borrowings),
    });
    await tx.wait();
    fetchData();
  }

  useEffect(() => {
    if (contract && account) {
      fetchData();
    }
  }, [contract, account]);

  return (
    <div style={{ padding: "20px" }}>
      <h1>💰 Loan Machine DApp</h1>

      <button onClick={connectWallet}>
        {account ? `Conectado: ${account.slice(0, 6)}...` : "Conectar Metamask"}
      </button>

      <h3>Saldo do contrato: {contractBalance} ETH</h3>

      <div>
        <h3>Doar</h3>
        <input
          type="text"
          placeholder="Valor em ETH"
          value={donationAmount}
          onChange={(e) => setDonationAmount(e.target.value)}
        />
        <button onClick={donate}>Doar</button>
      </div>

      <div>
        <h3>Pegar Emprestado</h3>
        <input
          type="text"
          placeholder="Valor em ETH"
          value={borrowAmount}
          onChange={(e) => setBorrowAmount(e.target.value)}
        />
        <button onClick={borrow}>Emprestar</button>
      </div>

      <div>
        <h3>Reembolsar</h3>
        <button onClick={repay}>Pagar Dívida</button>
      </div>

      <div>
        <h3>Meus Dados</h3>
        <p>Doações: {userStats.donations || 0} ETH</p>
        <p>Empréstimos: {userStats.borrowings || 0} ETH</p>
        <p>Pode pegar emprestado agora? {userStats.canBorrowNow ? "Sim" : "Não"}</p>
      </div>
    </div>
  );
}

export default App;
