import { useCallback } from 'react';
import { eventSystem } from './EventSystem';
import { extractErrorMessage, getErrorType } from './errorMapping';

export function useToast(provider, contract) { // NEW: Accept provider/contract for decoding
  const showToast = useCallback((message, type = 'info', options = {}) => {
    eventSystem.emit('showToast', {
      message,
      type,
      isError: type === 'error',
      duration: options.duration || 5000,
      ...options
    });
  }, []);

  const showError = useCallback(async (error, options = {}) => {
    const message = await extractErrorMessage(error, provider, contract.interface);
    const type = getErrorType(error);
    showToast(message, type, options);
  }, [showToast, provider, contract]);

  const showSuccess = useCallback((message, options = {}) => {
    showToast(message, 'success', options);
  }, [showToast]);

  const showWarning = useCallback((message, options = {}) => {
    showToast(message, 'warning', options);
  }, [showToast]);

  const showInfo = useCallback((message, options = {}) => {
    showToast(message, 'info', options);
  }, [showToast]);

  // UPDATED: Async, awaits decode
  const handleContractError = useCallback(async (error, context = '') => {
    console.error(`Erro em ${context}:`, error);
    await showError(error); // Awaits inside showError
  }, [showError]);

  return {
    showToast,
    showError,
    showSuccess,
    showWarning,
    showInfo,
    handleContractError
  };
}