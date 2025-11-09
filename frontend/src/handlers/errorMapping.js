import { decodeContractError } from './contractErrorDecoder'; // New import

// Keep your maps here if not in decoder (but I moved them there for simplicity)

// Async wrapper (must await in components)
export async function extractErrorMessage(error, provider, iface) {
  if (!error) return 'Erro desconhecido ocorreu';

  if (typeof error === 'string') {
    // Direct string handling (unchanged)
    // ... (keep your original string logic if needed)
    return error;
  }

  if (!provider || !iface) {
    console.warn('Provider or interface missing for decoding');
    return error.message || 'Erro inesperado';
  }

  return await decodeContractError(provider, iface, error);
}

// Keep unchanged
export function getErrorType(error) {
  const errorMsg = typeof error === 'string' ? error : error.message || '';
  const lowerMsg = errorMsg.toLowerCase();

  if (lowerMsg.includes('rejected') || 
      lowerMsg.includes('insufficient') ||
      lowerMsg.includes('minimum') ||
      lowerMsg.includes('invalid') ||
      lowerMsg.includes('required') ||
      lowerMsg.includes('cover') ||
      lowerMsg.includes('percentage') ||
      lowerMsg.includes('not found') ||
      lowerMsg.includes('wait')) {
    return 'warning';
  }

  if (lowerMsg.includes('failed') || 
      lowerMsg.includes('reverted') ||
      lowerMsg.includes('not available') ||
      lowerMsg.includes('already') ||
      lowerMsg.includes('expired') ||
      lowerMsg.includes('unauthorized') ||
      lowerMsg.includes('exceeds') ||
      lowerMsg.includes('maximum') ||
      lowerMsg.includes('only') ||
      lowerMsg.includes('cancelled')) {
    return 'error';
  }

  return 'error';
}

export function getKnownSelectors() {
  // From decoder or your map
  return Object.keys(SELECTOR_MESSAGES);
}

export function getKnownErrorNames() {
  return Object.keys(ERROR_MESSAGES);
}

console.log('Error mapping updated with decoder');