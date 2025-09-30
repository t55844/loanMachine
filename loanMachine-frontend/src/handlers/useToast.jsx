// hooks/useToast.js
import { useState, useCallback } from 'react';

export function useToast() {
  const [toast, setToast] = useState({ show: false, message: '', type: 'error' });

  const showToast = useCallback((message, type = 'error') => {
    setToast({ show: true, message, type });
    
    // Auto hide after 5 seconds
    setTimeout(() => {
      setToast(prev => ({ ...prev, show: false }));
    }, 5000);
  }, []);

  const hideToast = useCallback(() => {
    setToast(prev => ({ ...prev, show: false }));
  }, []);

  const handleContractError = useCallback((error, context = '') => {
    console.error(`Contract error in ${context}:`, error);
    
    const message = error.message?.toLowerCase() || '';
    
    // Map your custom errors
    if (message.includes('insufficientfunds')) {
      showToast('The contract does not have sufficient funds', 'error');
    } else if (message.includes('invalidamount')) {
      showToast('The specified amount is invalid', 'error');
    } else if (message.includes('minimumdonationrequired')) {
      showToast('Minimum donation requirement not met', 'error');
    } else if (message.includes('borrownotexpired')) {
      showToast('Previous borrowing period has not expired yet', 'error');
    } else if (message.includes('invalidcoveragepercentage')) {
      showToast('Coverage percentage must be between 1-100', 'error');
    } else if (message.includes('overcoverage')) {
      showToast('Cannot cover more than remaining percentage', 'error');
    } else if (message.includes('loannotavailable')) {
      showToast('This loan is no longer available', 'error');
    } else if (message.includes('insufficientdonationbalance')) {
      showToast('Your donation balance is insufficient', 'error');
    } else if (message.includes('noactiveborrowing')) {
      showToast('No active borrowing found', 'error');
    } else if (message.includes('repaymentexceedsborrowed')) {
      showToast('Repayment exceeds borrowed amount', 'error');
    } else if (message.includes('invalidparcelscount')) {
      showToast('Invalid number of parcels', 'error');
    } else if (message.includes('user rejected')) {
      showToast('Transaction cancelled', 'warning');
    } else {
      showToast(error.message || 'Transaction failed', 'error');
    }
  }, [showToast]);

  return {
    toast,
    showToast,
    hideToast,
    handleContractError
  };
}