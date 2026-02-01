import { ethers } from 'ethers';

// Your existing maps (copied here for convenience—merge with errorMapping if preferred)
const SELECTOR_MESSAGES = {
  '0xde99459f': 'LoanMachine_InvalidAmount: O valor fornecido é inválido',
  '0x13e73a29': 'LoanMachine_InsufficientFunds: Fundos insuficientes para esta transação (verifique saldo disponível no contrato)',
  '0x4b4e72f5': 'LoanMachine_MinimumDonationRequired: Valor da doação está abaixo do requisito mínimo',
  '0xead1ff82': 'LoanMachine_BorrowNotExpired: Empréstimo anterior deve ser pago ou expirado antes de emprestar novamente',
  '0xaac62d72': 'LoanMachine_InvalidCoveragePercentage: Porcentagem de cobertura inválida fornecida (deve ser 71-100%)',
  '0xded62aaf': 'LoanMachine_OverCoverage: Porcentagem de cobertura excede o máximo permitido',
  '0x186ebdb5': 'LoanMachine_LoanNotAvailable: Nenhum empréstimo disponível no momento',
  '0x37bc0490': 'LoanMachine_MaxLoanRequisitionPendingReached: Máximo de 3 requisições de empréstimo pendentes atingido',
  '0x56a620f8': 'LoanMachine_InsufficientDonationBalance: Saldo de doação insuficiente para esta ação',
  '0xcc2f71e0': 'LoanMachine_NoActiveBorrowing: Nenhum empréstimo ativo encontrado para esta carteira',
  '0x465b18f9': 'LoanMachine_RepaymentExceedsBorrowed: Valor do pagamento excede o valor emprestado',
  '0x235f466a': 'LoanMachine_InvalidParcelsCount: Número inválido de parcelas especificado',
  '0xa5a0e740': 'LoanMachine_ExcessiveDonation: Valor da doação excede o limite máximo',
  '0xb85dc7a6': 'LoanMachine_TokenTransferFailed: Transferência de token falhou - verifique aprovação e saldo',
  '0x5b1a00fa': 'LoanMachine_MemberIdOrWalletInvalid: ID do membro inválido ou carteira não vinculada. Por favor, verifique seu registro de membro.',
  '0x165baeef': 'LoanMachine_WalletAlreadyVinculated: Carteira já vinculada a um membro',
  '0x9881a565': 'LoanMachine_ParcelAlreadyPaid: Esta parcela já foi paga',
  '0x6925556a': 'LoanMachine_MinimumPercentageCover: Porcentagem mínima de cobertura não atingida',
  '0x9dd88a4e': 'LoanMachine_InsufficientWithdrawableBalance: Saldo sacável insuficiente',
  '0xf6d35245': 'LoanMachine_CheckIntervalNotYetPassed: Intervalo de verificação ainda não passou. Por favor, aguarde antes de realizar esta ação.',
  '0xde01143a': 'LoanMachine_NotLoanRequisitionCreator: Apenas o criador da requisição de empréstimo pode realizar esta ação.',
  '0xcfcb839e': 'LoanMachine_RequisitionAlreadyActive: Requisição de empréstimo já está ativa e não pode ser modificada.',
  '0x09b15068': 'LoanMachine_RequisitionNotFound: Requisição de empréstimo não encontrada.',
  '0x6938db79': 'LoanMachine_OnlyBorrowerCanCancelRequisition: Apenas o mutuário pode cancelar a requisição de empréstimo.',
  '0x29f68793': 'LoanMachine_RequisitionNotCancellable: Requisição de empréstimo não pode ser cancelada em seu estado atual.',
  '0x9398721d': 'LoanMachine_RequisitionAlreadyFullyCovered: Requisição de empréstimo já está totalmente coberta e não pode ser cancelada.',
  '0x6770d3df': 'LoanMachine_IntervalOfPaymentAboveLimit: Intervalo de pagamento excede o limite permitido (máx. 30 dias).',
  '0x0d266952': 'ReputationSystem_MemberIdOrWalletInvalid: ID do membro inválido ou carteira não vinculada',
  '0xb9d4ab66': 'ReputationSystem_WalletAlreadyVinculated: Carteira já vinculada a um membro',
  '0xa1b8c408': 'ReputationSystem_UnauthorizedCaller: Chamador não autorizado a realizar esta ação',
  '0x0ad153d1': 'ReputationSystem_ActiveElectionExists: Uma eleição ativa já existe',
  '0xac840993': 'ReputationSystem_ElectionNotActive: A eleição não está ativa',
  '0xd8524fa3': 'ReputationSystem_MemberAlreadyVoted: Membro já votou nesta eleição',
  '0x1677c8ba': 'ReputationSystem_InvalidCandidate: Candidato inválido para eleição',
  '0xfe88dfc5': 'ReputationSystem_NoCandidates: Sem candidatos na eleição',
  '0xe94b5834': 'ReputationSystem_ElectionExpired: A eleição expirou',
  '0x2dab5e1c': 'ReputationSystem_WalletAlreadyLinkedToAnotherMember: Carteira já vinculada a outro membro',
  '0x08c379a0': 'Revert with string: Erro genérico do contrato - verifique parâmetros',
  '0x4e487b71': 'Panic code: Erro interno (overflow/underflow) - contate o desenvolvedor',
  '0x1bf0a150': 'LoanMachine_InsufficientFunds: Fundos insuficientes no contrato (doe mais USDT ou verifique saldo disponível)',
};

const ERROR_MESSAGES = {
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
  'LoanMachine_CheckIntervalNotYetPassed': 'Intervalo de verificação ainda não passou. Por favor, aguarde antes de realizar esta ação.',
  'LoanMachine_NotLoanRequisitionCreator': 'Apenas o criador da requisição de empréstimo pode realizar esta ação.',
  'LoanMachine_RequisitionAlreadyActive': 'Requisição de empréstimo já está ativa e não pode ser modificada.',
  'LoanMachine_RequisitionNotFound': 'Requisição de empréstimo não encontrada.',
  'LoanMachine_OnlyBorrowerCanCancelRequisition': 'Apenas o mutuário pode cancelar a requisição de empréstimo.',
  'LoanMachine_RequisitionNotCancellable': 'Requisição de empréstimo não pode ser cancelada em seu estado atual.',
  'LoanMachine_RequisitionAlreadyFullyCovered': 'Requisição de empréstimo já está totalmente coberta e não pode ser cancelada.',
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
  'user rejected transaction': 'Transação rejeitada pelo usuário',
  'insufficient funds': 'ETH insuficiente para taxas de gas',
  'execution reverted': 'Transação falhou - execução do contrato revertida',
  'nonce too low': 'Nonce da transação está muito baixo',
  'gas limit exceeded': 'Transação requer mais gas do que o fornecido',
  'UNPREDICTABLE_GAS_LIMIT': 'Falha na estimativa de gas - verifique saldo de ETH ou erro no contrato',
  'gas required exceeds allowance': 'Gas necessário excede o saldo disponível - adicione ETH de teste'
};

// Recursive finder for nested revert data
function findNestedData(obj) {
  if (!obj) return null;
  if (typeof obj.data === 'string' && obj.data.startsWith('0x') && obj.data.length >= 10) return obj.data;
  if (typeof obj.reasonData === 'string' && obj.reasonData.startsWith('0x') && obj.reasonData.length >= 10) return obj.reasonData;
  for (const key of ['error', 'originalError', 'info', 'innerError', 'tx', 'transaction']) {
    if (obj[key]) {
      const found = findNestedData(obj[key]);
      if (found) return found;
    }
  }
  if (obj.body) {
    try {
      const parsed = JSON.parse(obj.body);
      const found = findNestedData(parsed);
      if (found) return found;
    } catch {}
  }
  // Stringify fallback for "data":"0x..."
  if (typeof obj === 'object') {
    const str = JSON.stringify(obj);
    const match = str.match(/"data"\s*:\s*"([0-9a-fA-F]{8})"/i);
    if (match) return '0x' + match[1].toLowerCase();
  }
  return null;
}

export async function decodeContractError(provider, iface, error) {
  //console.log('Decoding error:', error);

  let revertData = findNestedData(error) || error.data;

  // Replay if no data but reverted tx
  if (!revertData && error.code === 'CALL_EXCEPTION' && error.receipt?.status === 0) {
    let tx = error.transaction || error.tx;
    if (!tx || !tx.data) {
      tx = await provider.getTransaction(error.receipt.hash);
    }
    if (tx && tx.data) {
      try {
        // Replay at block before tx (reverts same way, captures data)
        await provider.call({
          from: tx.from,
          to: tx.to,
          data: tx.data,
          value: tx.value ?? 0n,
          gasLimit: 30_000_000n
        }, error.receipt.blockNumber - 1);
      } catch (replayErr) {
        revertData = findNestedData(replayErr) || replayErr.data;
        //console.log('Replay revert data:', revertData);
      }
    }
  }

  if (revertData && revertData.startsWith('0x')) {
    try {
      // Decode with ABI (best: gets name + args)
      const parsed = iface.parseError(revertData);
      let baseMsg = ERROR_MESSAGES[parsed.name] || parsed.name;
      const argsStr = parsed.args && parsed.args.length ? ` (${parsed.args.map(arg => arg.toString()).join(', ')})` : '';
      return `${baseMsg}${argsStr}`;
    } catch (parseErr) {
      // Fallback to selector
      const selector = revertData.slice(0, 10).toLowerCase();
      return SELECTOR_MESSAGES[selector] || `Unknown revert (selector: ${selector})`;
    }
  }

  // Fallback
  return error.shortMessage || error.message || 'Execution reverted (no data)';
}