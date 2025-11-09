import { useState, useEffect } from "react";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";
import { eventSystem } from "../handlers/EventSystem";

function Repay({ account, contract }) {
  const [amount, setAmount] = useState("");
  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { provider } = useWeb3(); // NEW: Get provider

  async function handleRepay() {
    if (!account || !amount) {
      alert("Por favor, conecte e insira um valor");
      return;
    }

    const value = ethers.utils.parseEther(amount); // FIXED: Use utils.parseEther
    
    showTransactionModal(
      {
        method: "repay",
        params: [],
        value: value.toString()
      },
      {
        type: 'repay',
        amount: amount
      }
    );
  }

  async function confirmTransaction(transactionData) {
    try {
      const value = ethers.BigNumber.from(transactionData.value);
      const tx = await contract.repay({
        value: value
      });
      await tx.wait();
      
      alert(`Pagamento de ${amount} ETH bem-sucedido!`);
      setAmount("");
    } catch (err) {
      //console.error(err);
      alert("Erro ao processar pagamento");
      throw err;
    }
  }

  return (
    <div className="repay-block">
      <input
        type="number"
        min={0}
        step="0.01"
        placeholder="Quantidade em ETH"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="repay-input"
      />

      <button onClick={handleRepay} className="repay-button" disabled={!account || !amount}>
        Pagar
      </button>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default Repay;