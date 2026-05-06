// Translates contract revert selectors into user-friendly messages.
//
// Every custom error in the Solidity contracts has a 4-byte selector equal to
// the first 4 bytes of keccak256("ErrorName()"). When a call reverts, alloy
// returns those bytes in the error payload. We match them here.

use alloy::primitives::Bytes;

/// Try to translate a revert payload into a friendly Portuguese message.
/// Returns None if the selector doesn't match a known contract error — the
/// caller should fall back to the raw alloy error in that case.
pub fn translate_revert(data: &[u8]) -> String {
    if data.len() < 4 {
        return "Erro desconhecido (payload vazio).".to_string();
    }
    let sel = [data[0], data[1], data[2], data[3]];
    match sel {
        // ── Reputation / Elections (RS_) ──────────────────────
        [0x1e, 0x7e, 0xc6, 0x03] => "ID de membro ou carteira inválido.".into(),
        [0xf9, 0x44, 0x05, 0x51] => "Esta carteira já está vinculada.".into(),
        [0xee, 0x0c, 0xfd, 0xe8] => "Esta carteira já está associada a outro membro.".into(),
        [0x62, 0x97, 0xdd, 0x4e] => "Já existe uma eleição ativa.".into(),
        [0x2e, 0x36, 0x7a, 0xef] => "Nenhuma eleição ativa no momento.".into(),
        [0xde, 0x6b, 0x8f, 0xb6] => "Este membro já votou nesta eleição.".into(),
        [0x70, 0xdc, 0xb1, 0x5d] => "Candidato inválido.".into(),
        [0x18, 0x30, 0x4b, 0x1b] => "Não há candidatos na eleição.".into(),

        // ── Multisig admin ────────────────────────────────────
        [0xd8, 0x9d, 0x05, 0x20] => "Apenas administradores podem executar esta ação.".into(),
        [0x86, 0x13, 0x08, 0x49] => "Este endereço já é administrador.".into(),
        [0xe1, 0xaa, 0xb7, 0x3b] => "Número insuficiente de administradores.".into(),
        [0xe9, 0x46, 0xad, 0xc6] => "Limite de assinaturas (threshold) inválido.".into(),
        [0x32, 0x43, 0xbb, 0x79] => "Esta proposta já foi executada.".into(),
        [0x6a, 0x17, 0x07, 0x9f] => "Você já confirmou esta proposta.".into(),
        [0x37, 0x1e, 0x2c, 0xc3] => "Proposta não encontrada.".into(),

        // ── Wallet approval / moderator ───────────────────────
        [0xc6, 0xd4, 0x31, 0xc1] => "Pedido de aprovação de carteira não encontrado.".into(),
        [0x31, 0xd7, 0xd4, 0x65] => "Esta aprovação de carteira já foi executada.".into(),
        [0xf3, 0x9a, 0xa3, 0x32] => "Apenas o moderador eleito pode executar esta ação.".into(),
        [0x42, 0x42, 0x64, 0x21] => "Administrador ainda não foi proposto.".into(),

        // ── Withdrawals ───────────────────────────────────────
        [0x26, 0x31, 0xd2, 0x1e] => "O saque ainda não pode ser executado (aguarde o prazo de 48h).".into(),
        [0x7e, 0xea, 0x2b, 0x43] => "Este saque foi bloqueado.".into(),
        [0xa7, 0xc0, 0xca, 0x2b] => "Este saque já foi executado.".into(),
        [0xd9, 0xbc, 0xe5, 0xd0] => "Apenas quem pediu o saque pode executá-lo.".into(),
        [0x1d, 0xa5, 0x97, 0xea] => "Apenas admin ou moderador pode executar esta ação.".into(),

        // ── Coop lifecycle / membership ───────────────────────
        [0x47, 0x6e, 0xbc, 0x32] => "Cooperativa já inicializada.".into(),
        [0x63, 0xfa, 0x89, 0x7b] => "Carteira não aprovada pela cooperativa.".into(),
        [0x7f, 0x1a, 0x48, 0xc5] => "Esta carteira já é membro da cooperativa.".into(),
        [0x67, 0xf7, 0x6b, 0xa4] => "Código de acesso inválido.".into(),
        [0x81, 0xe2, 0x31, 0xf6] => "Esta cooperativa não está ativa.".into(),

        // ── Financial ops ─────────────────────────────────────
        [0x0d, 0x84, 0xc0, 0xf0] => "Valor inválido.".into(),
        [0xf8, 0xa1, 0x7f, 0xc4] => "Fundos insuficientes.".into(),
        [0x1b, 0xf0, 0xa1, 0x50] => "Prazo do empréstimo ainda não expirou.".into(),
        [0x28, 0x19, 0x76, 0x0f] => "Porcentagem de cobertura inválida.".into(),
        [0x11, 0xd9, 0x35, 0x51] => "Cobertura acima do limite permitido.".into(),
        [0x72, 0x7c, 0xd2, 0x12] => "Empréstimo não disponível.".into(),
        [0xe1, 0x93, 0x5a, 0x36] => "Limite máximo de empréstimos pendentes atingido.".into(),
        [0x60, 0x44, 0xd4, 0xdd] => "Saldo de doações insuficiente.".into(),
        [0xc4, 0x96, 0x7c, 0xf3] => "Nenhum empréstimo ativo.".into(),
        [0x0a, 0x7d, 0x92, 0xaa] => "Quantidade de parcelas inválida.".into(),
        [0x7a, 0x97, 0xd7, 0x1f] => "Falha na transferência de tokens.".into(),
        [0x58, 0x4d, 0xaf, 0x25] => "ID de membro ou carteira inválido.".into(),
        [0x1c, 0xc5, 0x3e, 0x44] => "Porcentagem mínima de cobertura não atingida.".into(),
        [0xdb, 0xee, 0x9d, 0x4f] => "Saldo disponível para saque insuficiente.".into(),
        [0xc6, 0x71, 0xf0, 0x79] => "Apenas o tomador pode cancelar esta requisição.".into(),
        [0xe0, 0x47, 0x4f, 0x8a] => "Esta requisição não pode ser cancelada.".into(),
        [0x3e, 0xa1, 0xd7, 0x08] => "Requisição já totalmente coberta.".into(),
        [0x36, 0x31, 0xf3, 0x1d] => "Intervalo de pagamento acima do limite.".into(),

        // ── Registry ──────────────────────────────────────────
        [0x68, 0x33, 0x15, 0x20] => "Apenas o admin da plataforma pode registrar cooperativas.".into(),
        [0x47, 0x2a, 0x45, 0x6d] => "Cooperativa não encontrada no registro.".into(),
        [0xf0, 0x83, 0x02, 0x6c] => "Nome da cooperativa não pode ser vazio.".into(),
        [0xc3, 0x7b, 0x9b, 0x52] => "Esta cooperativa já está registrada.".into(),

        _ => format!(
            "Erro desconhecido (0x{:02x}{:02x}{:02x}{:02x}).",
            sel[0], sel[1], sel[2], sel[3]
        ),
    }
}

/// Extract the 4-byte selector payload from an alloy error, if any.
/// Works with the common shape of `TransportError(ErrorResp { data: ... })`.
pub fn extract_revert_data(err: &dyn std::error::Error) -> Option<Bytes> {
    // The payload is nested inside the error's Display form as hex.
    // For structured access, cast to alloy's specific error types.
    // Simplest approach that works across alloy versions: scan Debug output.
    let s = format!("{:?}", err);

    // Look for 0x-prefixed 8 hex chars (= 4 bytes = selector)
    let start = s.find("RawValue(\"0x")?;
    let rest  = &s[start + "RawValue(\"0x".len()..];
    let end   = rest.find('"')?;
    let hex   = &rest[..end];

    if hex.len() < 8 {
        return None;
    }
    hex::decode(hex).ok().map(Bytes::from)
}

/// Convenience: turn any blockchain error into a friendly string,
/// falling back to the underlying error's display if no selector matched.
pub fn friendly_from_error(err: &dyn std::error::Error) -> String {
    if let Some(bytes) = extract_revert_data(err) {
        return translate_revert(&bytes);
    }
    err.to_string()
}