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

  const ModalWrapper = ({ onConfirm }) => {
    const handleConfirm = async (transactionData) => {
      try {
        // Close modal immediately when user confirms
        hideModal();
        // Execute the transaction
        await onConfirm(transactionData);
      } catch (error) {
        // Error handling is done by the onConfirm function itself
        // No need to show modal for errors since you have toast notifications
        console.error("Transaction error:", error);
      }
    };

    return (
      <GasCostModal
        isOpen={showModal}
        onClose={hideModal}
        onConfirm={handleConfirm}
        transactionData={pendingTransaction}
        transactionContext={transactionContext}
        transactionStatus={'pending'} // Always show as pending since we close immediately
      />
    );
  };

  return {
    showTransactionModal,
    hideModal,
    ModalWrapper
  };
}