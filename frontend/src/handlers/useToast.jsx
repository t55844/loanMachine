import { useCallback } from 'react';
import { eventSystem } from '../handlers/EventSystem';
import { extractErrorMessage, getErrorType } from '../handlers/errorMapping';

export function useToast() {
  const showToast = useCallback((message, type = 'info', options = {}) => {
    // Use the event system to show toasts
    eventSystem.emit('showToast', {
      message,
      type,
      isError: type === 'error',
      duration: options.duration || 5000,
      ...options
    });
  }, []);

  const showError = useCallback((error, options = {}) => {
    const message = extractErrorMessage(error);
    const type = getErrorType(error);
    return showToast(message, type, options);
  }, [showToast]);

  const showSuccess = useCallback((message, options = {}) => {
    return showToast(message, 'success', options);
  }, [showToast]);

  const showWarning = useCallback((message, options = {}) => {
    return showToast(message, 'warning', options);
  }, [showToast]);

  const showInfo = useCallback((message, options = {}) => {
    return showToast(message, 'info', options);
  }, [showToast]);

  const handleContractError = useCallback((error, context = '') => {
    console.error(`Erro em ${context}:`, error);
    showError(error);
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