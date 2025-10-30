// handlers/errorMapping.js

// Map Solidity error selectors to human-readable messages
export const SELECTOR_MESSAGES = {
  // ReputationSystem errors
  '0x0ad153d1': 'ReputationSystem_ActiveElectionExists: An active election already exists',
  '0xe94b5834': 'ReputationSystem_ElectionExpired: The election has expired',
  '0xac840993': 'ReputationSystem_ElectionNotActive: The election is not active',
  '0x1677c8ba': 'ReputationSystem_InvalidCandidate: Invalid candidate for election',
  '0xd8524fa3': 'ReputationSystem_MemberAlreadyVoted: Member has already voted in this election',
  '0x0d266952': 'ReputationSystem_MemberIdOrWalletInvalid: Invalid member ID or wallet not vinculated',
  '0xfe88dfc5': 'ReputationSystem_NoCandidates: No candidates in the election',
  '0xa1b8c408': 'ReputationSystem_UnauthorizedCaller: Caller is not authorized to perform this action',
  '0x2dab5e1c': 'ReputationSystem_WalletAlreadyLinkedToAnotherMember: Wallet is already linked to another member',
  '0xb9d4ab66': 'ReputationSystem_WalletAlreadyVinculated: Wallet is already linked to a member',

  // LoanMachine errors - UPDATED with new errors
  '0xead1ff82': 'LoanMachine_BorrowNotExpired: Previous loan must be repaid or expired before borrowing again',
  '0xa5a0e740': 'LoanMachine_ExcessiveDonation: Donation amount exceeds maximum limit',
  '0x56a620f8': 'LoanMachine_InsufficientDonationBalance: Insufficient donation balance for this action',
  '0x13e73a29': 'LoanMachine_InsufficientFunds: Insufficient funds for this transaction',
  '0x9dd88a4e': 'LoanMachine_InsufficientWithdrawableBalance: Insufficient withdrawable balance',
  '0xde99459f': 'LoanMachine_InvalidAmount: The amount provided is invalid',
  '0xaac62d72': 'LoanMachine_InvalidCoveragePercentage: Invalid coverage percentage provided',
  '0x235f466a': 'LoanMachine_InvalidParcelsCount: Invalid number of parcels specified',
  '0x186ebdb5': 'LoanMachine_LoanNotAvailable: No loans available at the moment',
  '0x37bc0490': 'LoanMachine_MaxLoanRequisitionPendingReached: Maximum 3 pending loan requisitions reached',
  '0x5b1a00fa': 'LoanMachine_MemberIdOrWalletInvalid: Invalid member ID or wallet not vinculated. Please check your member registration.',
  '0x4b4e72f5': 'LoanMachine_MinimumDonationRequired: Donation amount is below the minimum requirement',
  '0x6925556a': 'LoanMachine_MinimumPercentageCover: Minimum percentage cover not reached',
  '0xcc2f71e0': 'LoanMachine_NoActiveBorrowing: No active borrowing found for this wallet',
  '0xded62aaf': 'LoanMachine_OverCoverage: Coverage percentage exceeds maximum allowed',
  '0x9881a565': 'LoanMachine_ParcelAlreadyPaid: This parcel has already been paid',
  '0x465b18f9': 'LoanMachine_RepaymentExceedsBorrowed: Repayment amount exceeds borrowed amount',
  '0xb85dc7a6': 'LoanMachine_TokenTransferFailed: Token transfer failed - check allowance and balance',
  '0x165baeef': 'LoanMachine_WalletAlreadyVinculated: Wallet is already linked to a member',
  '0xe1935a36': 'LoanMachine_MaxLoanRequisitionPendingReached: Max Loan Requisition Pending Reached',
  
  // NEW LoanMachine errors from your stack
  '0x0c42a0c9': 'LoanMachine_CheckIntervalNotYetPassed: Check interval not yet passed. Please wait before performing this action.',
  '0x5b1a00fa': 'LoanMachine_NotLoanRequisitionCreator: Only the loan requisition creator can perform this action.',
  '0x4b4e72f5': 'LoanMachine_RequisitionAlreadyActive: Loan requisition is already active and cannot be modified.',
  '0x6925556a': 'LoanMachine_RequisitionNotFound: Loan requisition not found.',
  '0x9881a565': 'LoanMachine_OnlyBorrowerCanCancelRequisition: Only the borrower can cancel the loan requisition.',
  '0x465b18f9': 'LoanMachine_RequisitionNotCancellable: Loan requisition cannot be cancelled in its current state.',
  '0xb85dc7a6': 'LoanMachine_RequisitionAlreadyFullyCovered: Loan requisition is already fully covered and cannot be cancelled.',
};

// Map error names to messages (for cases where name is available)
export const ERROR_MESSAGES = {
  // LoanMachine Custom errors - UPDATED with new errors
  'LoanMachine_InvalidAmount': 'The amount provided is invalid',
  'LoanMachine_InsufficientFunds': 'Insufficient funds for this transaction',
  'LoanMachine_MinimumDonationRequired': 'Donation amount is below the minimum requirement',
  'LoanMachine_BorrowNotExpired': 'Previous loan must expired before borrowing again',
  'LoanMachine_InvalidCoveragePercentage': 'Invalid coverage percentage provided',
  'LoanMachine_OverCoverage': 'Coverage percentage exceeds maximum allowed',
  'LoanMachine_LoanNotAvailable': 'No loans available at the moment',
  'LoanMachine_MaxLoanRequisitionPendingReached': 'Maximum 3 pending loan requisitions reached',
  'LoanMachine_InsufficientDonationBalance': 'Insufficient donation balance for this action',
  'LoanMachine_NoActiveBorrowing': 'No active borrowing found for this wallet',
  'LoanMachine_RepaymentExceedsBorrowed': 'Repayment amount exceeds borrowed amount',
  'LoanMachine_InvalidParcelsCount': 'Invalid number of parcels specified',
  'LoanMachine_ExcessiveDonation': 'Donation amount exceeds maximum limit',
  'LoanMachine_TokenTransferFailed': 'Token transfer failed - check allowance and balance',
  'LoanMachine_MemberIdOrWalletInvalid': 'Invalid member ID or wallet not vinculated. Please check your member registration.',
  'LoanMachine_WalletAlreadyVinculated': 'Wallet is already linked to a member',
  'LoanMachine_ParcelAlreadyPaid': 'This parcel has already been paid',
  'LoanMachine_InsufficientWithdrawableBalance': 'Insufficient withdrawable balance',
  'LoanMachine_MinimumPercentageCover': 'Minimum percentage cover not reached',
  
  // NEW LoanMachine errors
  'LoanMachine_CheckIntervalNotYetPassed': 'Check interval not yet passed. Please wait before performing this action.',
  'LoanMachine_NotLoanRequisitionCreator': 'Only the loan requisition creator can perform this action.',
  'LoanMachine_RequisitionAlreadyActive': 'Loan requisition is already active and cannot be modified.',
  'LoanMachine_RequisitionNotFound': 'Loan requisition not found.',
  'LoanMachine_OnlyBorrowerCanCancelRequisition': 'Only the borrower can cancel the loan requisition.',
  'LoanMachine_RequisitionNotCancellable': 'Loan requisition cannot be cancelled in its current state.',
  'LoanMachine_RequisitionAlreadyFullyCovered': 'Loan requisition is already fully covered and cannot be cancelled.',

  // ReputationSystem Custom errors
  'ReputationSystem_ActiveElectionExists': 'An active election already exists',
  'ReputationSystem_ElectionExpired': 'The election has expired',
  'ReputationSystem_ElectionNotActive': 'The election is not active',
  'ReputationSystem_InvalidCandidate': 'Invalid candidate for election',
  'ReputationSystem_MemberAlreadyVoted': 'Member has already voted in this election',
  'ReputationSystem_NoCandidates': 'No candidates in the election',
  'ReputationSystem_UnauthorizedCaller': 'Caller is not authorized to perform this action',
  'ReputationSystem_WalletAlreadyLinkedToAnotherMember': 'Wallet is already linked to another member',
  'ReputationSystem_MemberIdOrWalletInvalid': 'Invalid member ID or wallet not vinculated',
  'ReputationSystem_WalletAlreadyVinculated': 'Wallet is already linked to a member',

  // Common Ethereum errors
  'user rejected transaction': 'Transaction was rejected by user',
  'insufficient funds': 'Insufficient ETH for gas fees',
  'execution reverted': 'Transaction failed - contract execution reverted',
  'nonce too low': 'Transaction nonce is too low',
  'gas limit exceeded': 'Transaction requires more gas than provided'
};

// Function to extract error message from various error formats
export function extractErrorMessage(error) {
  if (!error) return 'Unknown error occurred';

  console.log('Raw error received:', error);

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

  console.log('Error message:', errorMessage);
  console.log('Error data:', errorData);
  console.log('Error code:', errorCode);

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
    console.log('Error reason:', error.reason);
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
  return errorMessage || 'An unexpected error occurred. Please try again.';
}

// Helper function to extract error from blockchain data (selector-based)
function extractErrorFromData(data) {
  if (!data) return null;

  console.log('Extracting from data:', data);

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
      console.log(`Found selector ${selector} - ${SELECTOR_MESSAGES[selector]}`);
      return SELECTOR_MESSAGES[selector];
    }

    if (selector) {
      return `Contract error (selector: ${selector})`;
    }
  }

  return null;
}

// Helper function to extract error from message string (more reliable)
function extractErrorFromMessage(message) {
  if (!message || typeof message !== 'string') return null;

  console.log('Extracting from message:', message);

  const lowerMessage = message.toLowerCase();

  // Pattern 1: Direct error name in message (most reliable)
  for (const errorName of Object.keys(ERROR_MESSAGES)) {
    if (message.includes(errorName)) {
      console.log(`Found direct error name: ${errorName}`);
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
console.log('Error mapping updated with new LoanMachine and ReputationSystem errors');