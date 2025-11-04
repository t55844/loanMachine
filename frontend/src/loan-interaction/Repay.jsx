import { useState } from "react";
import { ethers } from "ethers";
import { useGasCostModal } from "../handlers/useGasCostModal";

function Repay({ account, contract }) {
  const [amount, setAmount] = useState("");
  const { showTransactionModal, ModalWrapper } = useGasCostModal();

  async function handleRepay() {
    if (!account || !contract || !amount) {
      alert("Por favor, conecte e insira um valor");
      return;
    }

    const value = ethers.utils.parseEther(amount);
    
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
      console.error(err);
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