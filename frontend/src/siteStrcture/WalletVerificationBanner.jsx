import { useState, useEffect } from "react";
import { fetchWalletMember } from "../graphql-frontend-query";
import { useWeb3 } from "../Web3Context";

export default function WalletVerificationBanner() {
  const [isVerified, setIsVerified] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  const { account } = useWeb3();

  useEffect(() => {
    if (account) {
      checkWalletVerification();
    } else {
      setIsVerified(null);
    }
  }, [account]);

  async function checkWalletVerification() {
    if (!account) return;

    setLoading(true);
    setError("");

    try {
      const memberData = await fetchWalletMember(account);
      console.log(memberData)
      setIsVerified(!!memberData);
    } catch (e) {
      console.error("Error checking wallet verification:", e);
      setError("Failed to verify wallet");
      setIsVerified(false);
    } finally {
      setLoading(false);
    }
  }

  async function handleRetry() {
    if (!account) return;

    setLoading(true);
    setError("");

    try {
      const memberData = await fetchWalletMember(account);
      const verified = !!memberData;
      setIsVerified(verified);
      
      // Only reload if the retry was successful
      if (verified) {
        window.location.reload();
      }
    } catch (e) {
      console.error("Error checking wallet verification:", e);
      setError("Failed to verify wallet");
      setIsVerified(false);
    } finally {
      setLoading(false);
    }
  }

  // Don't show anything if no wallet connected or still loading
  if (!account || loading) {
    return null;
  }

  // Don't show warning if wallet is verified
  if (isVerified) {
    return null;
  }

  return (
    <div className="wallet-verification-banner">
      <div className="warning-content">
        <div className="warning-icon">⚠️</div>
        <div className="warning-text">
          <strong>Wallet Not Vinculated</strong>
          <span>This wallet is not linked to any member account. Some features may be limited.</span>
          {error && <span className="error-small">Verification failed: {error}</span>}
        </div>
        <button 
          onClick={handleRetry} 
          className="retry-button"
          disabled={loading}
        >
          {loading ? "Checking..." : "Retry"}
        </button>
      </div>
    </div>
  );
}