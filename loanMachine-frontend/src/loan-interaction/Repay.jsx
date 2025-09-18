import { useState } from "react";
import { ethers } from "ethers";
import abi from "../abi/loanMachine.json";

const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;
const RPC_URL = import.meta.env.VITE_RPC_URL;

const HARDHAT_WALLETS = [
  { name: "0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199", privateKey: "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e" },
  { name: "0xdD2FD4581271e230360230F9337D5c0430Bf44C0", privateKey: "0xde9be858da4a475276426320d5e9262ecfc3ba460bfac56360bfa6c4c28b4ee0" },
  { name: "0xbDA5747bFD65F08deb54cb465eB87D40e51B197E", privateKey: "0x689af8efa8c651a91ad287602527f3af2fe9f6501a7ac4b061667b5a93e037fd" },
  { name: "0x2546BcD3c84621e976D8185a91A922aE77ECEc30", privateKey: "0xea6c44ac03bff858b476bba40716402b03e41b8e97e276d1baec7c37d42484a0" }
];

function Repay() {
  const [selectedWalletIndex, setSelectedWalletIndex] = useState("");
  const [amount, setAmount] = useState("");

  async function handleRepay() {
    if (selectedWalletIndex === "" || !amount) {
      alert("Selecione uma carteira e insira um valor");
      return;
    }

    try {
      const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
      const wallet = HARDHAT_WALLETS[selectedWalletIndex];
      const signer = new ethers.Wallet(wallet.privateKey, provider);
      const contract = new ethers.Contract(CONTRACT_ADDRESS, abi.abi, signer);

      const tx = await contract.repay({
        value: ethers.utils.parseEther(amount)
      });
      await tx.wait();

      alert(`Empréstimo de ${amount} ETH devolvido pelo ${wallet.name}!`);
    } catch (err) {
      console.error(err);
      alert("Erro ao devolver empréstimo");
    }
  }

  return (
    <div className="repay-block">
      <select
        value={selectedWalletIndex}
        onChange={(e) => setSelectedWalletIndex(e.target.value)}
        className="repay-select"
      >
        <option value="">Selecione um wallet</option>
        {HARDHAT_WALLETS.map((wallet, index) => (
          <option key={index} value={index}>
            {wallet.name}
          </option>
        ))}
      </select>

      <input
        type="number"
        min={0}
        step="0.01"
        placeholder="Valor em ETH"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="repay-input"
      />

      <button onClick={handleRepay} className="repay-button">
        Pagar
      </button>
    </div>
  );
}

export default Repay;
