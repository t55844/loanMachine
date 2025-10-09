// hooks/useGasCostModal.js
import { useState } from 'react';
import GasCostModal from '../handlers/GasCostModal';

export function useGasCostModal() {
  const [showModal, setShowModal] = useState(false);
  const [pendingTransaction, setPendingTransaction] = useState(null);
  const [transactionContext, setTransactionContext] = useState(null);
  const [transactionStatus, setTransactionStatus] = useState('pending'); // 'pending', 'processing', 'success', 'error'

  const showTransactionModal = (transactionData, context = null) => {
    setPendingTransaction(transactionData);
    setTransactionContext(context);
    setTransactionStatus('pending');
    setShowModal(true);
  };

  const hideModal = () => {
    setShowModal(false);
    setPendingTransaction(null);
    setTransactionContext(null);
    setTransactionStatus('pending');
  };

  const ModalWrapper = ({ onConfirm }) => {
    const handleConfirm = async (transactionData) => {
      try {
        setTransactionStatus('processing');
        await onConfirm(transactionData);
        setTransactionStatus('success');
        // DON'T auto-close - let user close manually
      } catch (error) {
        setTransactionStatus('error');
        // DON'T auto-close - let user close manually
      }
    };

    return (
      <GasCostModal
        isOpen={showModal}
        onClose={hideModal}
        onConfirm={handleConfirm}
        transactionData={pendingTransaction}
        transactionContext={transactionContext}
        transactionStatus={transactionStatus}
      />
    );
  };

  return {
    showTransactionModal,
    hideModal,
    ModalWrapper
  };
}