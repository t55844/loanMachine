// handlers/errorMapping.js

// Map Solidity error selectors to human-readable messages
export const SELECTOR_MESSAGES = {
  // LoanMachine errors
  '0x5aad290f': 'MaxLoanRequisitionPendingReached: Maximum 3 pending loan requisitions reached. Cover or complete an existing one first.',
  '0x8dae2a4f': 'MemberIdOrWalletInvalid: Invalid member ID or wallet not vinculated. Please vinculate your wallet first.',
  '0x0feab579': 'ExcessiveDonation: Donation amount exceeds maximum limit',
  '0x021176a5': 'InsufficientDonationBalance: Insufficient donation balance for this action',
  '0x6d0a6a9c': 'InsufficientFunds: Insufficient funds for this transaction',
  '0x2af956d8': 'InsufficientWithdrawableBalance: Insufficient withdrawable balance',
  '0x2f4f6d6d': 'InvalidAmount: The amount provided is invalid',
  '0x5b64a1c1': 'InvalidCoveragePercentage: Invalid coverage percentage provided',
  '0x3a7b1e31': 'InvalidParcelsCount: Invalid number of parcels specified',
  '0x8dae2a4f': 'LoanNotAvailable: No loans available at the moment',
  '0x5aad290f': 'MinimumDonationRequired: Donation amount is below the minimum requirement',
  '0x8dae2a4f': 'MinimumPercentageCover: Minimum percentage cover not reached',
  '0x5aad290f': 'NoActiveBorrowing: No active borrowing found for this wallet',
  '0x0feab579': 'OverCoverage: Coverage percentage exceeds maximum allowed',
  '0x5aad290f': 'ParcelAlreadyPaid: This parcel has already been paid',
  '0x021176a5': 'RepaymentExceedsBorrowed: Repayment amount exceeds borrowed amount',
  '0x6d0a6a9c': 'TokenTransferFailed: Token transfer failed - check allowance and balance',
  '0x2af956d8': 'WalletAlreadyVinculated: Wallet is already linked to a member',
  '0x5aad290f': 'BorrowNotExpired: Previous loan must be repaid or expired before borrowing again',

  // ReputationSystem errors
  '0x5aad290f': 'ActiveElectionExists: An active election already exists',
  '0x8dae2a4f': 'ElectionExpired: The election has expired',
  '0x0feab579': 'ElectionNotActive: The election is not active',
  '0x021176a5': 'InvalidCandidate: Invalid candidate for election',
  '0x6d0a6a9c': 'MemberAlreadyVoted: Member has already voted in this election',
  '0x2f4f6d6d': 'NoCandidates: No candidates in the election',
  '0x5b64a1c1': 'UnauthorizedCaller: Caller is not authorized to perform this action',
  '0x3a7b1e31': 'WalletAlreadyLinkedToAnotherMember: Wallet is already linked to another member',
};

// Map error names to messages (for cases where name is available)
export const ERROR_MESSAGES = {
  // LoanMachine Custom errors
  'InvalidAmount': 'The amount provided is invalid',
  'InsufficientFunds': 'Insufficient funds for this transaction',
  'MinimumDonationRequired': 'Donation amount is below the minimum requirement',
  'BorrowNotExpired': 'Previous loan must be repaid or expired before borrowing again',
  'InvalidCoveragePercentage': 'Invalid coverage percentage provided',
  'OverCoverage': 'Coverage percentage exceeds maximum allowed',
  'LoanNotAvailable': 'No loans available at the moment',
  'MaxLoanRequisitionPendingReached': 'Maximum pending loan requisitions reached',
  'InsufficientDonationBalance': 'Insufficient donation balance for this action',
  'NoActiveBorrowing': 'No active borrowing found for this wallet',
  'RepaymentExceedsBorrowed': 'Repayment amount exceeds borrowed amount',
  'InvalidParcelsCount': 'Invalid number of parcels specified',
  'ExcessiveDonation': 'Donation amount exceeds maximum limit',
  'TokenTransferFailed': 'Token transfer failed - check allowance and balance',
  'MemberIdOrWalletInvalid': 'Invalid member ID or wallet not vinculated. Please check your member registration.',
  'WalletAlreadyVinculated': 'Wallet is already linked to a member',
  'ParcelAlreadyPaid': 'This parcel has already been paid',
  'InsufficientWithdrawableBalance': 'Insufficient withdrawable balance',
  'MinimumPercentageCover': 'Minimum percentage cover not reached',

  // ReputationSystem Custom errors
  'ActiveElectionExists': 'An active election already exists',
  'ElectionExpired': 'The election has expired',
  'ElectionNotActive': 'The election is not active',
  'InvalidCandidate': 'Invalid candidate for election',
  'MemberAlreadyVoted': 'Member has already voted in this election',
  'NoCandidates': 'No candidates in the election',
  'UnauthorizedCaller': 'Caller is not authorized to perform this action',
  'WalletAlreadyLinkedToAnotherMember': 'Wallet is already linked to another member',

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
  
  // If it's already a string message
  if (typeof error === 'string') {
    return ERROR_MESSAGES[error] || error;
  }
  
  // Check for custom error selector in data
  let data = null;
  if (error.data) {
    data = error.data;
  } else if (error.error && error.error.data) {
    data = error.error.data;
  } else if (error.reason && error.reason.includes('return data')) {
    const match = error.reason.match(/return data: (0x[0-9a-f]{8})/);
    if (match) {
      data = match[1];
    }
  }
  
  // Handle the case where blockchain sends only the hash/selector
  if (data && typeof data === 'string') {
    // Check if it's a full revert data (starts with 0x and has selector + encoded data)
    if (data.startsWith('0x') && data.length >= 10) {
      const selector = data.toLowerCase().slice(0, 10); // First 4 bytes (8 hex chars + 0x)
      
      // Try to find the selector in our mapping
      if (SELECTOR_MESSAGES[selector]) {
        return SELECTOR_MESSAGES[selector];
      }
      
      // If we have a longer data string but no selector match, try to extract potential error message
      if (data.length > 10) {
        return `Contract error: ${data}`;
      }
      
      return `Unrecognized custom error (${selector})`;
    }
    
    // Handle case where we might get just the 8-character selector without 0x
    if (data.length === 8 && /^[0-9a-f]{8}$/i.test(data)) {
      const selector = '0x' + data.toLowerCase();
      return SELECTOR_MESSAGES[selector] || `Unrecognized custom error (${selector})`;
    }
  }
  
  // If it's an Error object with message
  if (error.message) {
    const errorMsg = error.message.toLowerCase();
    
    // Check for common error patterns
    for (const [key, message] of Object.entries(ERROR_MESSAGES)) {
      if (errorMsg.includes(key.toLowerCase())) {
        return message;
      }
    }
    
    // Check for revert with reason
    if (errorMsg.includes('reverted')) {
      const match = errorMsg.match(/reverted with reason string '([^']+)'/);
      if (match && match[1]) {
        return ERROR_MESSAGES[match[1]] || match[1];
      }
      
      const customMatch = errorMsg.match(/reverted with custom error '([^']+)'/);
      if (customMatch && customMatch[1]) {
        return ERROR_MESSAGES[customMatch[1]] || customMatch[1];
      }
      
      // Try to extract selector from revert message
      const selectorMatch = errorMsg.match(/0x[0-9a-f]{8}/);
      if (selectorMatch) {
        const selector = selectorMatch[0].toLowerCase();
        if (SELECTOR_MESSAGES[selector]) {
          return SELECTOR_MESSAGES[selector];
        }
      }
    }
    
    return ERROR_MESSAGES[errorMsg] || error.message;
  }
  
  // If it's an object with error data
  if (error.error?.message) {
    return extractErrorMessage(error.error.message);
  }
  
  if (error.data?.message) {
    return extractErrorMessage(error.data.message);
  }
  
  return 'An unexpected error occurred. Please try again.';
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
      lowerMsg.includes('percentage')) {
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
      lowerMsg.includes('maximum')) {
    return 'error';
  }
  
  return 'error'; // default to error
}

// Helper function to get all known error selectors
export function getKnownSelectors() {
  return Object.keys(SELECTOR_MESSAGES);
}

// Helper function to get all known error names
export function getKnownErrorNames() {
  return Object.keys(ERROR_MESSAGES);
}