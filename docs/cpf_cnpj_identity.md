# CPF / CNPJ — Validation & Identity Pipeline

Documents (CPF for individuals, CNPJ for companies) are the link between a person and their on-chain identity. They are **never stored** anywhere — only a salted hash leaves the validation layer.

---

## 1. The Two Document Kinds

```
DocKind::Cpf   — Brazilian individual tax ID.  11 bare digits.
DocKind::Cnpj  — Brazilian company tax ID.     14 bare characters (alphanumeric since RF 2026).
```

The enum is defined in `loan_machine_models/src/requests.rs` and crosses the wire with every form submission that involves a document.

---

## 2. Client Side — `DocumentInput` component

**File:** `loan_machine_web/src/components/cpf_cnpj/doc_input_snipet.rs`

The component owns the toggle (CPF ↔ CNPJ) and the text input. It does three things inline as the user types, before anything reaches the server:

### 2a. Input filtering (`on_document_input`)

```
CPF  → keeps only ASCII digits and the separators . - /
CNPJ → keeps ASCII alphanumeric + separators . - /
         uppercases every letter
```

Characters that don't match are stripped in-place so the DOM value and the Leptos signal stay in sync.

### 2b. Length cap

`max_input_len()` returns the maximum **formatted** character count:

| Kind  | max_input_len | Example formatted          |
|-------|---------------|----------------------------|
| CPF   | 14            | `123.456.789-09`           |
| CNPJ  | 18            | `12.ABC.345/0001-88`       |

The input uses `take(max_len)` on every keystroke, so the user cannot type past the limit.

### 2c. Pre-submit validation

Before dispatching the server action the parent component checks:

```rust
let document_ok = !doc.trim().is_empty()
    && dk.bare_char_count(&doc) == dk.max_digits();
```

`bare_char_count` counts alphanumeric characters only (strips separators). `max_digits` is 11 for CPF and 14 for CNPJ. If this guard fails the form shows a field-level error and never calls the server.

---

## 3. Transport

The cleaned, formatted string (e.g. `"12.ABC.345/0001-88"`) travels to the server as the `document: String` parameter of a server function alongside `doc_kind: DocKind`.

Two server functions accept this pair:

| Server fn                    | File                             | Purpose                     |
|------------------------------|----------------------------------|-----------------------------|
| `prepare_first_vinculation`  | `server_fns/vinculation.rs`      | bind wallet to cooperative  |
| `prepare_create_coop`        | `server_fns/create_coop.rs`      | founder identity for deploy |

The server fn immediately hands both values to the logic layer. No server fn ever touches the document after that point.

---

## 4. Server Side — Validation

**File:** `loan_machine_core/src/services/identity/mod.rs`

### 4a. CPF — `validate_cpf`

```
1. Strip non-digits (remove . - /)
2. Reject if length ≠ 11           → CpfLength
3. Reject if all digits identical  → CpfAllSame
4. Mod-11 check digit (two rounds)
     weights₁ = [10, 9, 8, 7, 6, 5, 4, 3, 2]
     weights₂ = [11, 10, 9, 8, 7, 6, 5, 4, 3, 2]
     remainder = sum % 11
     digit     = 0 if remainder < 2, else 11 − remainder
                                    → CpfCheckDigits on mismatch
5. Return bare 11-digit string
```

### 4b. CNPJ — `validate_cnpj`

```
1. strip_cnpj_formatting
     keeps only ASCII alphanumeric, uppercases
     e.g. "12.ABC.345/0001-88" → "12ABC345000188"
2. Reject if length ≠ 14           → CnpjLength
3. Reject if all chars identical   → CnpjAllSame
4. Reject if chars[12..] not digits → CnpjCheckDigits
     (check digit positions are always numeric per RF spec)
5. Map every character to its weight value (see table below)
6. Mod-11 check digit (two rounds)
     W1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]       → D13
     W2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]    → D14
     remainder = sum % 11
     digit     = 0 if remainder < 2, else 11 − remainder
                                    → CnpjCheckDigits on mismatch
7. Return bare 14-character string
```

#### Character value table (SERPRO / RF 2026 spec)

The value for any character is `ASCII decimal − 48`:

| Char | ASCII | Value |   | Char | ASCII | Value |
|------|-------|-------|---|------|-------|-------|
| `0`  | 48    | 0     |   | `A`  | 65    | 17    |
| `1`  | 49    | 1     |   | `B`  | 66    | 18    |
| …    | …     | …     |   | …    | …     | …     |
| `9`  | 57    | 9     |   | `Z`  | 90    | 42    |

For digits the formula is identical to `face value`; for letters it gives A=17 … Z=42. **Not** A=10 … Z=35.

Source: [SERPRO — Cálculo DV CNPJ Alfanumérico](https://www.serpro.gov.br/menu/noticias/videos/calculodvcnpjalfanaumerico.pdf)

---

## 5. Identity Hashing

After validation returns the bare string (e.g. `"12ABC345000188"`) it is immediately hashed:

```
memberId = keccak256(COOP_SALT ++ doc_bytes)
```

`COOP_SALT` is a 32-byte secret loaded from the environment (`COOP_SALT` env var, 64 hex chars). `doc_bytes` is the UTF-8 encoding of the bare validated string.

The resulting `FixedBytes<32>` is the on-chain member identifier — it is safe to store and log. The raw document string is dropped by the caller immediately after hashing.

**File:** `hash_member_id` in `loan_machine_core/src/services/identity/mod.rs`

---

## 6. End-to-End Flow Summary

```
browser input  →  DocumentInput filters + uppercases
               →  bare_char_count guard before submit
               →  server fn receives (DocKind, formatted string)
               →  validate_cpf / validate_cnpj strips separators, runs mod-11
               →  hash_member_id  →  FixedBytes<32> member ID
               →  sent to blockchain / subgraph (raw document never leaves the server)
```
