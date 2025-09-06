export default function WalletConnect({ setAccount }) {
  async function connectWallet() {
    if (window.ethereum) {
      const accounts = await window.ethereum.request({ method: "eth_requestAccounts" });
      setAccount(accounts[0]);
    } else {
      alert("MetaMask not found!");
    }
  }

 return (
  <button onClick={connectWallet} className="button button-blue">
    Connect Wallet
  </button>
);

}
