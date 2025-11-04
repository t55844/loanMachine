import { useState } from "react";
// Removed fetchWalletMember as it's handled by refreshMemberData
import { useWeb3 } from "../Web3Context"; 

export default function WalletVerificationBanner() {
    // Local state for the *Retry Button* only
    const [retryLoading, setRetryLoading] = useState(false); 
    const [error, setError] = useState("");
    
    // Get the global loading state and member data from context
    const { 
        account, 
        member, 
        refreshMemberData,
        loading: web3Loading // Rename global loading state
    } = useWeb3();

    const isVerified = member?.hasVinculation;

    async function handleRetry() {
        if (!account) return;

        setRetryLoading(true); // Only set local loading for the button
        setError("");

        try {
            await refreshMemberData(); // This updates the global 'member' state
            // The component will automatically re-render and hide the banner
            // because 'isVerified' will become true.
            
        } catch (e) {
            console.error("Erro ao verificar carteira:", e);
            setError("Falha ao verificar carteira");
        } finally {
            setRetryLoading(false);
        }
    }

    // 1. Initial Suppression (Fixes the state clearing flash on reload)
    // Hide the banner if:
    // a) No account is connected OR
    // b) The Web3 provider is in the middle of connecting (global loading)
    if (!account || web3Loading) {
        return null;
    }

    // 2. Hide if already verified
    if (isVerified) {
        return null;
    }

    // 3. Show the banner (with local button loading state)
    return (
        <div className="wallet-verification-banner">
            <div className="warning-content">
                <div className="warning-icon">⚠️</div>
                <div className="warning-text">
                    <strong>Carteira Não Vinculada</strong>
                    <span>Esta carteira não está vinculada a nenhuma conta de membro. Alguns recursos podem ser limitados.</span>
                    {error && <span className="error-small">Falha na verificação: {error}</span>}
                </div>
                <button 
                    onClick={handleRetry} 
                    className="retry-button-banner"
                    disabled={retryLoading} // Use local retryLoading state
                >
                    {retryLoading ? "Verificando..." : "Tentar Novamente"}
                </button>
            </div>
        </div>
    );
}