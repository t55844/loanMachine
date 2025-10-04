// hooks/useGasCostModal.js
import { useState } from 'react';
import GasCostModal from '../handlers/GasCostModal';

export function useGasCostModal() {
  const [showModal, setShowModal] = useState(false);
  const [pendingTransaction, setPendingTransaction] = useState(null);
  const [transactionContext, setTransactionContext] = useState(null);

  const showTransactionModal = (transactionData, context = null) => {
    setPendingTransaction(transactionData);
    setTransactionContext(context);
    setShowModal(true);
  };

  const hideModal = () => {
    setShowModal(false);
    setPendingTransaction(null);
    setTransactionContext(null);
  };

  const ModalWrapper = ({ account, contract, onConfirm }) => {
    const handleConfirm = async (transactionData) => {
      try {
        await onConfirm(transactionData);
        // Close modal after successful confirmation
        hideModal();
      } catch (error) {
        // Don't close modal on error - let user retry or cancel
        console.error('Transaction failed:', error);
      }
    };

    return (
      <GasCostModal
        isOpen={showModal}
        onClose={hideModal}
        onConfirm={handleConfirm}
        transactionData={pendingTransaction}
        transactionContext={transactionContext}
        account={account}
        contract={contract}
      />
    );
  };

  return {
    showTransactionModal,
    hideModal,
    ModalWrapper
  };
}