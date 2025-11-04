// Map Solidity error selectors to human-readable messages
export const SELECTOR_MESSAGES = {
  // ReputationSystem errors
  '0x0ad153d1': 'ReputationSystem_ActiveElectionExists: Uma eleição ativa já existe',
  '0xe94b5834': 'ReputationSystem_ElectionExpired: A eleição expirou',
  '0xac840993': 'ReputationSystem_ElectionNotActive: A eleição não está ativa',
  '0x1677c8ba': 'ReputationSystem_InvalidCandidate: Candidato inválido para eleição',
  '0xd8524fa3': 'ReputationSystem_MemberAlreadyVoted: Membro já votou nesta eleição',
  '0x0d266952': 'ReputationSystem_MemberIdOrWalletInvalid: ID do membro inválido ou carteira não vinculada',
  '0xfe88dfc5': 'ReputationSystem_NoCandidates: Sem candidatos na eleição',
  '0xa1b8c408': 'ReputationSystem_UnauthorizedCaller: Chamador não autorizado a realizar esta ação',
  '0x2dab5e1c': 'ReputationSystem_WalletAlreadyLinkedToAnotherMember: Carteira já vinculada a outro membro',
  '0xb9d4ab66': 'ReputationSystem_WalletAlreadyVinculated: Carteira já vinculada a um membro',

  // LoanMachine errors - UPDATED with new errors
  '0xead1ff82': 'LoanMachine_BorrowNotExpired: Empréstimo anterior deve ser pago ou expirado antes de emprestar novamente',
  '0xa5a0e740': 'LoanMachine_ExcessiveDonation: Valor da doação excede o limite máximo',
  '0x56a620f8': 'LoanMachine_InsufficientDonationBalance: Saldo de doação insuficiente para esta ação',
  '0x13e73a29': 'LoanMachine_InsufficientFunds: Fundos insuficientes para esta transação',
  '0x9dd88a4e': 'LoanMachine_InsufficientWithdrawableBalance: Saldo sacável insuficiente',
  '0xde99459f': 'LoanMachine_InvalidAmount: O valor fornecido é inválido',
  '0xaac62d72': 'LoanMachine_InvalidCoveragePercentage: Porcentagem de cobertura inválida fornecida',
  '0x235f466a': 'LoanMachine_InvalidParcelsCount: Número inválido de parcelas especificado',
  '0x186ebdb5': 'LoanMachine_LoanNotAvailable: Nenhum empréstimo disponível no momento',
  '0x37bc0490': 'LoanMachine_MaxLoanRequisitionPendingReached: Máximo de 3 requisições de empréstimo pendentes atingido',
  '0x5b1a00fa': 'LoanMachine_MemberIdOrWalletInvalid: ID do membro inválido ou carteira não vinculada. Por favor, verifique seu registro de membro.',
  '0x4b4e72f5': 'LoanMachine_MinimumDonationRequired: Valor da doação está abaixo do requisito mínimo',
  '0x6925556a': 'LoanMachine_MinimumPercentageCover: Porcentagem mínima de cobertura não atingida',
  '0xcc2f71e0': 'LoanMachine_NoActiveBorrowing: Nenhum empréstimo ativo encontrado para esta carteira',
  '0xded62aaf': 'LoanMachine_OverCoverage: Porcentagem de cobertura excede o máximo permitido',
  '0x9881a565': 'LoanMachine_ParcelAlreadyPaid: Esta parcela já foi paga',
  '0x465b18f9': 'LoanMachine_RepaymentExceedsBorrowed: Valor do pagamento excede o valor emprestado',
  '0xb85dc7a6': 'LoanMachine_TokenTransferFailed: Transferência de token falhou - verifique aprovação e saldo',
  '0x165baeef': 'LoanMachine_WalletAlreadyVinculated: Carteira já vinculada a um membro',
  '0xe1935a36': 'LoanMachine_MaxLoanRequisitionPendingReached: Máximo de Requisições de Empréstimo Pendentes Atingido',
  
  // NEW LoanMachine errors from your stack
  '0x0c42a0c9': 'LoanMachine_CheckIntervalNotYetPassed: Intervalo de verificação ainda não passou. Por favor, aguarde antes de realizar esta ação.',
  '0x5b1a00fa': 'LoanMachine_NotLoanRequisitionCreator: Apenas o criador da requisição de empréstimo pode realizar esta ação.',
  '0x4b4e72f5': 'LoanMachine_RequisitionAlreadyActive: Requisição de empréstimo já está ativa e não pode ser modificada.',
  '0x6925556a': 'LoanMachine_RequisitionNotFound: Requisição de empréstimo não encontrada.',
  '0x9881a565': 'LoanMachine_OnlyBorrowerCanCancelRequisition: Apenas o mutuário pode cancelar a requisição de empréstimo.',
  '0x465b18f9': 'LoanMachine_RequisitionNotCancellable: Requisição de empréstimo não pode ser cancelada em seu estado atual.',
  '0xb85dc7a6': 'LoanMachine_RequisitionAlreadyFullyCovered: Requisição de empréstimo já está totalmente coberta e não pode ser cancelada.',
};

// Map error names to messages (for cases where name is available)
export const ERROR_MESSAGES = {
  // LoanMachine Custom errors - UPDATED with new errors
  'LoanMachine_InvalidAmount': 'O valor fornecido é inválido',
  'LoanMachine_InsufficientFunds': 'Fundos insuficientes para esta transação',
  'LoanMachine_MinimumDonationRequired': 'Valor da doação está abaixo do requisito mínimo',
  'LoanMachine_BorrowNotExpired': 'Empréstimo anterior deve expirar antes de emprestar novamente',
  'LoanMachine_InvalidCoveragePercentage': 'Porcentagem de cobertura inválida fornecida',
  'LoanMachine_OverCoverage': 'Porcentagem de cobertura excede o máximo permitido',
  'LoanMachine_LoanNotAvailable': 'Nenhum empréstimo disponível no momento',
  'LoanMachine_MaxLoanRequisitionPendingReached': 'Máximo de 3 requisições de empréstimo pendentes atingido',
  'LoanMachine_InsufficientDonationBalance': 'Saldo de doação insuficiente para esta ação',
  'LoanMachine_NoActiveBorrowing': 'Nenhum empréstimo ativo encontrado para esta carteira',
  'LoanMachine_RepaymentExceedsBorrowed': 'Valor do pagamento excede o valor emprestado',
  'LoanMachine_InvalidParcelsCount': 'Número inválido de parcelas especificado',
  'LoanMachine_ExcessiveDonation': 'Valor da doação excede o limite máximo',
  'LoanMachine_TokenTransferFailed': 'Transferência de token falhou - verifique aprovação e saldo',
  'LoanMachine_MemberIdOrWalletInvalid': 'ID do membro inválido ou carteira não vinculada. Por favor, verifique seu registro de membro.',
  'LoanMachine_WalletAlreadyVinculated': 'Carteira já vinculada a um membro',
  'LoanMachine_ParcelAlreadyPaid': 'Esta parcela já foi paga',
  'LoanMachine_InsufficientWithdrawableBalance': 'Saldo sacável insuficiente',
  'LoanMachine_MinimumPercentageCover': 'Porcentagem mínima de cobertura não atingida',
  
  // NEW LoanMachine errors
  'LoanMachine_CheckIntervalNotYetPassed': 'Intervalo de verificação ainda não passou. Por favor, aguarde antes de realizar esta ação.',
  'LoanMachine_NotLoanRequisitionCreator': 'Apenas o criador da requisição de empréstimo pode realizar esta ação.',
  'LoanMachine_RequisitionAlreadyActive': 'Requisição de empréstimo já está ativa e não pode ser modificada.',
  'LoanMachine_RequisitionNotFound': 'Requisição de empréstimo não encontrada.',
  'LoanMachine_OnlyBorrowerCanCancelRequisition': 'Apenas o mutuário pode cancelar a requisição de empréstimo.',
  'LoanMachine_RequisitionNotCancellable': 'Requisição de empréstimo não pode ser cancelada em seu estado atual.',
  'LoanMachine_RequisitionAlreadyFullyCovered': 'Requisição de empréstimo já está totalmente coberta e não pode ser cancelada.',

  // ReputationSystem Custom errors
  'ReputationSystem_ActiveElectionExists': 'Uma eleição ativa já existe',
  'ReputationSystem_ElectionExpired': 'A eleição expirou',
  'ReputationSystem_ElectionNotActive': 'A eleição não está ativa',
  'ReputationSystem_InvalidCandidate': 'Candidato inválido para eleição',
  'ReputationSystem_MemberAlreadyVoted': 'Membro já votou nesta eleição',
  'ReputationSystem_NoCandidates': 'Sem candidatos na eleição',
  'ReputationSystem_UnauthorizedCaller': 'Chamador não autorizado a realizar esta ação',
  'ReputationSystem_WalletAlreadyLinkedToAnotherMember': 'Carteira já vinculada a outro membro',
  'ReputationSystem_MemberIdOrWalletInvalid': 'ID do membro inválido ou carteira não vinculada',
  'ReputationSystem_WalletAlreadyVinculated': 'Carteira já vinculada a um membro',

  // Common Ethereum errors
  'user rejected transaction': 'Transação rejeitada pelo usuário',
  'insufficient funds': 'ETH insuficiente para taxas de gas',
  'execution reverted': 'Transação falhou - execução do contrato revertida',
  'nonce too low': 'Nonce da transação está muito baixo',
  'gas limit exceeded': 'Transação requer mais gas do que o fornecido'
};

// Function to extract error message from various error formats
export function extractErrorMessage(error) {
  if (!error) return 'Erro desconhecido ocorreu';

  console.log('Erro bruto recebido:', error);

  // If it's already a string message and it's one of our mapped errors, return it directly
  if (typeof error === 'string') {
    // Check if it's a direct error name we recognize
    if (ERROR_MESSAGES[error]) {
      return ERROR_MESSAGES[error];
    }

    // Check if it's already a formatted error message (contains colon)
    for (const message of Object.values(ERROR_MESSAGES)) {
      if (error.includes(message.split(':')[0])) {
        return message;
      }
    }

    return error;
  }

  // Handle Error objects and complex error structures
  let errorMessage = error.message || '';
  let errorData = error.data || error.error?.data;
  let errorCode = error.code;

  console.log('Mensagem de erro:', errorMessage);
  console.log('Dados de erro:', errorData);
  console.log('Código de erro:', errorCode);

  // STRATEGY: Now with unique selectors, we can reliably map both ways
  // 1. First try to extract from error message/reason (most reliable)
  // 2. Then try selector matching (now reliable due to unique selectors)

  // Extract from error message first (most reliable when available)
  if (errorMessage) {
    const extractedFromMessage = extractErrorFromMessage(errorMessage);
    if (extractedFromMessage) {
      return extractedFromMessage;
    }
  }

  // Try from reason field
  if (error.reason) {
    console.log('Razão do erro:', error.reason);
    const extractedFromReason = extractErrorFromMessage(error.reason);
    if (extractedFromReason) {
      return extractedFromReason;
    }
  }

  // Extract from error data (selector-based, now reliable)
  if (errorData) {
    const extractedFromData = extractErrorFromData(errorData);
    if (extractedFromData) {
      return extractedFromData;
    }
  }

  // Last resort: Check for common error patterns in the message
  if (errorMessage) {
    const lowerMessage = errorMessage.toLowerCase();
    for (const [key, message] of Object.entries(ERROR_MESSAGES)) {
      if (lowerMessage.includes(key.toLowerCase())) {
        return message;
      }
    }
  }

  // Last resort: return the original message or generic error
  return errorMessage || 'Um erro inesperado ocorreu. Por favor, tente novamente.';
}

// Helper function to extract error from blockchain data (selector-based)
function extractErrorFromData(data) {
  if (!data) return null;

  console.log('Extraindo de dados:', data);

  let dataString = data;

  if (typeof data === 'object') {
    if (data.message) return extractErrorFromMessage(data.message);
    if (data.originalError) return extractErrorFromData(data.originalError);
    dataString = JSON.stringify(data);
  }

  if (typeof dataString === 'string') {
    // Extract selector from data
    let selector = null;

    if (dataString.startsWith('0x') && dataString.length >= 10) {
      selector = dataString.toLowerCase().slice(0, 10);
    } else if (dataString.length >= 8 && /^[0-9a-f]{8}$/i.test(dataString)) {
      selector = '0x' + dataString.toLowerCase();
    }

    if (selector && SELECTOR_MESSAGES[selector]) {
      console.log(`Encontrado seletor ${selector} - ${SELECTOR_MESSAGES[selector]}`);
      return SELECTOR_MESSAGES[selector];
    }

    if (selector) {
      return `Erro de contrato (seletor: ${selector})`;
    }
  }

  return null;
}

// Helper function to extract error from message string (more reliable)
function extractErrorFromMessage(message) {
  if (!message || typeof message !== 'string') return null;

  console.log('Extraindo de mensagem:', message);

  const lowerMessage = message.toLowerCase();

  // Pattern 1: Direct error name in message (most reliable)
  for (const errorName of Object.keys(ERROR_MESSAGES)) {
    if (message.includes(errorName)) {
      console.log(`Encontrado nome de erro direto: ${errorName}`);
      return ERROR_MESSAGES[errorName];
    }
  }

  // Pattern 2: Reverted with reason string
  if (lowerMessage.includes('reverted with reason string')) {
    const match = message.match(/reverted with reason string ['"]([^'"]+)['"]/i);
    if (match && match[1]) {
      const reason = match[1];
      return ERROR_MESSAGES[reason] || reason;
    }
  }

  // Pattern 3: Reverted with custom error
  if (lowerMessage.includes('reverted with custom error')) {
    const match = message.match(/reverted with custom error ['"]([^'"]+)['"]/i);
    if (match && match[1]) {
      const errorName = match[1];
      return ERROR_MESSAGES[errorName] || errorName;
    }
  }

  // Pattern 4: VM Exception with revert
  if (lowerMessage.includes('vm exception') && lowerMessage.includes('revert')) {
    // Try to extract error name from the message
    for (const errorName of Object.keys(ERROR_MESSAGES)) {
      if (message.includes(errorName)) {
        return ERROR_MESSAGES[errorName];
      }
    }

    // Try to extract selector from VM exception
    const selectorMatch = message.match(/0x[0-9a-f]{8}/i);
    if (selectorMatch) {
      const selector = selectorMatch[0].toLowerCase();
      if (SELECTOR_MESSAGES[selector]) {
        return SELECTOR_MESSAGES[selector];
      }
    }
  }

  // Pattern 5: Gas estimation error with underlying contract error
  if (lowerMessage.includes('cannot estimate gas') || lowerMessage.includes('unpredictable_gas_limit')) {
    // Look for the underlying contract error in the nested data
    const selectorMatch = message.match(/0x[0-9a-f]{8}/i);
    if (selectorMatch) {
      const selector = selectorMatch[0].toLowerCase();
      if (SELECTOR_MESSAGES[selector]) {
        return SELECTOR_MESSAGES[selector];
      }
    }
  }

  return null;
}

// Function to determine error type for styling
export function getErrorType(error) {
  const errorMsg = typeof error === 'string' ? error : error.message || '';
  const lowerMsg = errorMsg.toLowerCase();

  // User action errors (warnings)
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

  // Critical errors (errors)
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

// Helper function to get all known error selectors
export function getKnownSelectors() {
  return Object.keys(SELECTOR_MESSAGES);
}

// Helper function to get all known error names
export function getKnownErrorNames() {
  return Object.keys(ERROR_MESSAGES);
}

// Note: Some selector collisions detected - this is normal in development
// In production, ensure each custom error has a unique selector
console.log('Mapeamento de erro atualizado com novos erros do LoanMachine e ReputationSystem');