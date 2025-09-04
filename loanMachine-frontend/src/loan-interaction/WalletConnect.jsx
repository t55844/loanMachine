import { ethers } from "ethers";

function WalletConnect({ setAccount }) {
  async function connectWallet() {
    if (window.ethereum) {
      const provider = new ethers.providers.Web3Provider(window.ethereum);
      await provider.send("eth_requestAccounts", []);
      const signer = provider.getSigner();
      const address = await signer.getAddress();
      setAccount(address);
    } else {
      alert("MetaMask não detectado!");
    }
  }

  return (
    <div className="mb-4">
      <button onClick={connectWallet} className="bg-blue-500 text-white px-4 py-2 rounded">
        Conectar Carteira
      </button>
    </div>
  );
}

export default WalletConnect;
