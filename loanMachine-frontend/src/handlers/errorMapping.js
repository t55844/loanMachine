// handlers/errorMapping.js

// Map Solidity error selectors to human-readable messages (add 0x5aad290f for your error)
export const SELECTOR_MESSAGES = {
  '0x5aad290f': 'MaxLoanRequisitionPendingReached: Maximum 3 pending loan requisitions reached. Cover or complete an existing one first.',
  '0x8dae2a4f': 'Wallet not vinculated or invalid member ID. Please vinculate your wallet first.',  // Your existing one (if needed)
  // Add other selectors as needed (e.g., from keccak256 of error names)
  // For example:
  // '0x0feab579': 'InvalidAmount',
  // '0x021176a5': 'InsufficientFunds',
  // etc. You can add more by computing keccak256 of error names
};

// Map error names to messages (for cases where name is available)
export const ERROR_MESSAGES = {
  // Custom errors
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
  
  if (data && typeof data === 'string' && data.startsWith('0x') && data.length === 10) {
    const selector = data.toLowerCase();
    return SELECTOR_MESSAGES[selector] || `Unrecognized custom error (${data})`;
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
      lowerMsg.includes('invalid')) {
    return 'warning';
  }
  
  // Critical errors (errors)
  if (lowerMsg.includes('failed') || 
      lowerMsg.includes('reverted') ||
      lowerMsg.includes('not available') ||
      lowerMsg.includes('already')) {
    return 'error';
  }
  
  return 'error'; // default to error
}