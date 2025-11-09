import { useState } from "react";
import { ethers } from "ethers";
import { useWeb3 } from "../Web3Context"; // Assume this gives provider, contract
import { useToast } from "../handlers/useToast";
import Toast from "../handlers/Toast";

export default function LoanRequisitionForm({ 
  contract, 
  account, 
  onRequisitionCreated, 
  member 
}) {
  const [amount, setAmount] = useState("");
  const [minimumCoverage, setMinimumCoverage] = useState("80");
  const [parcelsQuantity, setParcelsQuantity] = useState("1");
  const [daysIntervalOfPayment, setDaysIntervalOfPayment] = useState("30");
  const [loading, setLoading] = useState(false);
  
  const { provider } = useWeb3(); // NEW: Get provider
  const { showToast, showSuccess, handleContractError } = useToast(provider, contract); // NEW: Pass them

  // UPDATED: Async handleSubmit, await in catch
  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!contract || !account) {
      showToast("Por favor, conecte ao nó Hardhat primeiro");
      return;
    }

    if (!member || !member.id) {
      showToast("Dados do membro não disponíveis. Por favor, verifique sua conexão com a carteira.");
      return;
    }
    
    setLoading(true);

    try {
      // FIXED: Use ethers.utils.parseUnits for v5 compatibility
      const amountWei = ethers.utils.parseUnits(amount, 6);
      const memberId = member.id;

      const tx = await contract.createLoanRequisition(
        amountWei,
        parseInt(minimumCoverage),
        parseInt(parcelsQuantity),
        memberId,
        parseInt(daysIntervalOfPayment)
      );
      
      await tx.wait();
      
      setAmount("");
      setMinimumCoverage("80");
      setDaysIntervalOfPayment("30");
      
      showSuccess("Requisição de empréstimo criada com sucesso!");
      
      if (onRequisitionCreated) {
        onRequisitionCreated();
      }
      
    } catch (err) {
      await handleContractError(err, "createLoanRequisition"); // NEW: Await
    } finally {
      setLoading(false);
    }
  };

  const hasMemberData = member && member.id;

  return (
    <>
      <Toast />
      
      <div style={{
        background: 'var(--bg-tertiary)',
        padding: '24px',
        borderRadius: '12px',
        border: '1px solid var(--border-color)',
        marginTop: '16px'
      }}>
        <h2>Criar Requisição de Empréstimo</h2>
        
        {!member && account && (
          <div style={{
            background: 'var(--warning-bg)',
            padding: '12px',
            borderRadius: '8px',
            marginBottom: '16px',
            border: '1px solid var(--warning-color)'
          }}>
            <p style={{ margin: 0, fontSize: '14px', color: 'var(--warning-color)' }}>
              ⚠️ Dados do membro não carregados. Por favor, verifique sua conexão com a carteira.
            </p>
          </div>
        )}
        
        <form onSubmit={handleSubmit}>
          <div style={{ marginBottom: '16px', textAlign: 'center' }}>
            <label htmlFor="amount" style={{display: 'block', marginBottom: '8px'}}>Valor do Empréstimo (USDT)</label>
            <input
              id="amount"
              type="number"
              step="0.01"
              min="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              className="wallet-input"
              placeholder="Insira o valor"
              required
              style={{width: '100%'}}
            />
          </div>

          <div style={{ display:'flex', justifyContent: 'space-around', flexWrap: 'wrap', gap: '16px' }}>
            <div style={{ minWidth: '120px', flex: '1', textAlign: 'left' }}>
              <label htmlFor="minimumCoverage" style={{display: 'block', marginBottom: '8px'}}>Cobertura Mínima (%)</label>
              <select
                id="minimumCoverage"
                value={minimumCoverage}
                onChange={(e) => setMinimumCoverage(e.target.value)}
                className="donate-select"
                style={{width: '100%'}}
                required
              >
                <option value="71">71%</option>
                <option value="75">75%</option>
                <option value="80">80%</option>
                <option value="85">85%</option>
                <option value="90">90%</option>
                <option value="95">95%</option>
                <option value="100">100%</option>
              </select>
            </div>
            
            <div style={{ minWidth: '120px', flex: '1', textAlign: 'left' }}>
              <label htmlFor="parcelsQuantity" style={{display: 'block', marginBottom: '8px'}}>Quantidade de Parcelas</label>
              <select
                id="parcelsQuantity"
                value={parcelsQuantity}
                onChange={(e) => setParcelsQuantity(e.target.value)}
                className="donate-select"
                style={{width: '100%'}}
                required
              >
                <option value="1">1</option>
                <option value="2">2</option>
                <option value="3">3</option>
                <option value="4">4</option>
                <option value="5">5</option>
                <option value="6">6</option>
                <option value="7">7</option>
                <option value="8">8</option>
                <option value="9">9</option>
                <option value="10">10</option>
                <option value="11">11</option>
                <option value="12">12</option>
              </select>
            </div>

            <div style={{ minWidth: '120px', flex: '1', textAlign: 'left' }}>
              <label htmlFor="daysIntervalOfPayment" style={{display: 'block', marginBottom: '8px'}}>Intervalo de Pagamento (Dias)</label>
              <select
                id="daysIntervalOfPayment"
                value={daysIntervalOfPayment}
                onChange={(e) => setDaysIntervalOfPayment(e.target.value)}
                className="donate-select"
                style={{width: '100%'}}
                required
              >
                <option value="1">1 Dia</option>
                <option value="5">5 Dias</option>
                <option value="10">10 Dias</option>
                <option value="15">15 Dias</option>
                <option value="20">20 Dias</option>
                <option value="25">25 Dias</option>
                <option value="30">30 Dias</option>
              </select>
            </div>
          </div>
          
          <button 
            type="submit" 
            className="borrow-button"
            disabled={loading || !hasMemberData}
            style={{width: '100%', marginTop: '16px'}}
          >
            {loading ? "Criando..." : !hasMemberData ? "Carteira Não Vinculada" : "Criar Requisição de Empréstimo"}
          </button>
        </form>
      </div>
    </>
  );
}