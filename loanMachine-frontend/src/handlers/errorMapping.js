// handlers/errorMapping.js

import { ethers } from 'ethers';
import LoanMachineABI from '../../../artifacts/contracts/LoanMachine.sol/LoanMachine.json';  // Import your ABI for decoding

const loanInterface = new ethers.utils.Interface(LoanMachineABI.abi);  // Cache Interface for parsing

// Map Solidity error selectors to human-readable messages (updated with all custom errors)
export const SELECTOR_MESSAGES = {
  '0x0feab579': 'InvalidAmount: The amount provided is invalid or zero.',
  '0x021176a5': 'InsufficientFunds: Not enough funds available for this loan.',
  '0xb5293b86': 'MinimumDonationRequired: You need to donate at least 1 USDT to borrow.',
  '0x03979cf4': 'BorrowNotExpired: Your previous loan must expire (7 days) before borrowing again.',
  '0x2999b8c6': 'InvalidCoveragePercentage: Coverage percentage must be between 1% and 100%.',
  '0x03b78666': 'OverCoverage: Total coverage cannot exceed 100%.',
  '0xc4043747': 'LoanNotAvailable: This loan requisition is not available for coverage.',
  '0x75046598': 'MaxLoanRequisitionPendingReached: Maximum 3 pending loan requisitions reached. Cover or complete an existing one first.',
  '0x06b7a68e': 'InsufficientDonationBalance: Not enough donations to cover this percentage.',
  '0x3fb5859c': 'NoActiveBorrowing: No active loan found for repayment.',
  '0x39881350': 'RepaymentExceedsBorrowed: Repayment amount exceeds the borrowed or parcel value.',
  '0xc7574b48': 'InvalidParcelsCount: Number of parcels must be between 1 and 12.',
  '0xa4e662bc': 'ExcessiveDonation: Donation amount exceeds the maximum limit of 5 USDT.',
  '0x45b29d56': 'TokenTransferFailed: USDT transfer failed—check your balance and allowance.',
  '0xe582ac72': 'MemberIdOrWalletInvalid: Invalid member ID or wallet not vinculated. Please vinculate your wallet first.',
  '0x23dbc337': 'WalletAlreadyVinculated: This wallet is already linked to a member.',
  '0x009ad0ab': 'ParcelAlreadyPaid: This parcel has already been paid.',
  // Legacy/fallback for old deploys
  '0x8dae2a4f': 'MaxLoanRequisitionPendingReached: Maximum 3 pending loan requisitions reached. (Legacy)',
  // Common Ethereum errors (non-custom)
  '0x4e487b71': 'Arithmetic overflow/underflow—try smaller amounts.',
  '0x08c379a0': 'Execution reverted—check transaction parameters.',
  // ... add more as needed
};

// Map error names to messages (for cases where name is available via ABI decoding)
export const ERROR_MESSAGES = {
  // Custom errors (fallback if selector not matched)
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
  
  // PRIORITY: Try ABI-based decoding with Interface (handles custom errors by name)
  try {
    let revertData = error.data?.data || error.error?.data || error.reason;
    if (revertData && typeof revertData === 'string' && revertData.startsWith('0x')) {
      // Clean up: Remove standard revert prefixes if present (e.g., 0x08c379a0 or 0x4e487b71)
      if (revertData.startsWith('0x08c379a0') || revertData.startsWith('0x4e487b71')) {
        revertData = '0x' + revertData.substring(10);  // Full data after prefix
      }
      
      // Parse with Interface—returns { name: 'MaxLoanRequisitionPendingReached', args: [] }
      const decoded = loanInterface.parseError(revertData);
      if (decoded && decoded.name) {
        const name = decoded.name;
        const argsStr = decoded.args && decoded.args.length > 0 ? `: ${decoded.args.join(', ')}` : '';
        return `${name}${argsStr}`;  // e.g., "MaxLoanRequisitionPendingReached"
      }
    }
  } catch (decodeErr) {
    console.warn('ABI decode failed, falling back to selector mapping:', decodeErr);
  }
  
  // FALLBACK: Check for custom error selector in data (manual mapping)
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

// utils/EventSystem.js
class EventSystem {
  constructor() {
    this.listeners = {};
  }

  on(event, callback) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(callback);
  }

  emit(event, data) {
    if (this.listeners[event]) {
      this.listeners[event].forEach(callback => callback(data));
    }
  }
}

export const eventSystem = new EventSystem();