// errorMapping.js (simplified, call the async decoder)
import { decodeContractError } from './contractErrorDecoder';

let contractInterface; // Set globally or pass in
let provider; // From Web3Context

export async function extractErrorMessage(error, _provider, _iface) {
  if (!error) return 'Erro desconhecido';
  provider = _provider || provider;
  contractInterface = _iface || contractInterface;
  if (!provider || !contractInterface) return error.message || 'Erro (falta provider/ABI)';

  return await decodeContractError(provider, contractInterface, error);
}

// Keep getErrorType, etc.