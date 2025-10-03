// hooks/useGasCostModal.js
import { useState } from 'react';
import GasCostModal from './GasCostModal';

export function useGasCostModal() {
  const [showModal, setShowModal] = useState(false);
  const [pendingTransaction, setPendingTransaction] = useState(null);

  const showTransactionModal = (transactionData) => {
    setPendingTransaction(transactionData);
    setShowModal(true);
  };

  const hideModal = () => {
    setShowModal(false);
    setPendingTransaction(null);
  };

  const ModalWrapper = ({ account, contract, onConfirm }) => (
    <GasCostModal
      isOpen={showModal}
      onClose={hideModal}
      onConfirm={onConfirm}
      transactionData={pendingTransaction}
      account={account}
      contract={contract}
    />
  );

  return {
    showTransactionModal,
    hideModal,
    ModalWrapper
  };
}