// utils/errorMapping.js

// Map Solidity error selectors to human-readable messages
export const ERROR_MESSAGES = {
  // Custom errors mapping
  'InvalidAmount': 'The amount provided is invalid',
  'InsufficientFunds': 'Insufficient funds for this transaction',
  'MinimumDonationRequired': 'Donation amount is below the minimum requirement',
  'BorrowNotExpired': 'Previous loan must be repaid or expired before borrowing again',
  'InvalidCoveragePercentage': 'Invalid coverage percentage provided',
  'OverCoverage': 'Coverage percentage exceeds maximum allowed',
  'LoanNotAvailable': 'No loans available at the moment',
  'InsufficientDonationBalance': 'Insufficient donation balance for this action',
  'NoActiveBorrowing': 'No active borrowing found for this wallet',
  'RepaymentExceedsBorrowed': 'Repayment amount exceeds borrowed amount',
  'InvalidParcelsCount': 'Invalid number of parcels specified',
  'ExcessiveDonation': 'Donation amount exceeds maximum limit',
  'TokenTransferFailed': 'Token transfer failed - check allowance and balance',
  'MemberIdOrWalletInvalid': 'Invalid member ID or wallet address',
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
  
  // If it's an Error object with message
  if (error.message) {
    const errorMsg = error.message.toLowerCase();
    
    // Check for common error patterns
    for (const [key, message] of Object.entries(ERROR_MESSAGES)) {
      if (errorMsg.includes(key.toLowerCase())) {
        return message;
      }
    }
    
    // Check for specific Solidity error patterns
    if (errorMsg.includes('reverted')) {
      // Try to extract the custom error name
      const match = errorMsg.match(/reverted with reason string '([^']+)'/);
      if (match && match[1]) {
        return ERROR_MESSAGES[match[1]] || match[1];
      }
      
      // Check for custom error selectors
      const customErrorMatch = errorMsg.match(/reverted with custom error '([^']+)'/);
      if (customErrorMatch && customErrorMatch[1]) {
        return ERROR_MESSAGES[customErrorMatch[1]] || customErrorMatch[1];
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
  
  return 'An unexpected error occurred';
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