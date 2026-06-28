// @coral-xyz/anchor: the TypeScript client for Anchor programs.
// Knows how to serialize instructions, send transactions, and deserialize
// on-chain accounts — all from the IDL that `anchor build` generates.
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

// LoanMachine is a TypeScript type generated at build time from the IDL
// (target/types/loan_machine.ts). It gives you full autocomplete and
// type-checking for every instruction and account in the program.
import { LoanMachine } from "../target/types/loan_machine";

// Keypair and PublicKey are from the base Solana web3.js SDK — they are
// chain primitives, not Anchor-specific.
import { Keypair, PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("loan_machine — initialize", () => {
  // AnchorProvider.env() reads the local surfpool RPC URL and the test
  // wallet keypair from environment variables set by `anchor test`.
  // This gives the test a signer (payer) and an RPC connection in one object.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // anchor.workspace.LoanMachine loads the program using the program ID
  // from Anchor.toml and the IDL from target/idl/loan_machine.json.
  // The cast `as Program<LoanMachine>` gives TypeScript the full typed API.
  const program = anchor.workspace.LoanMachine as Program<LoanMachine>;

  // Derive the GlobalState PDA locally — same seeds as in lib.rs:
  //   seeds = [b"global"], bump
  // `findProgramAddressSync` tries bumps from 255 downward until it finds
  // a pubkey that does NOT lie on the ed25519 curve (that's what makes it
  // a valid PDA — no one can sign for it, only the program can).
  // The returned bump is not needed here because the program stores it.
  const [globalStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("global")],
    program.programId
  );

  // A random pubkey standing in for a real USDT SPL mint.
  // We don't need the actual mint to work for this test — we just verify
  // the program stores whatever pubkey we pass.
  const mockUsdtMint = Keypair.generate().publicKey;

  it("creates GlobalState PDA with correct fields", async () => {
    // program.methods.initialize(mockUsdtMint) — constructs a typed
    // instruction builder. The argument maps to `usdt_mint: Pubkey` in lib.rs.
    //
    // .accounts({...}) — fills the accounts array required by the
    // `Initialize` context struct in lib.rs. Anchor validates at runtime
    // that all constraints (seeds, payer, etc.) are satisfied.
    //
    // .rpc() — serializes the instruction into a transaction, signs it
    // with provider.wallet (the test keypair), sends it to surfpool,
    // and waits for confirmation. Returns the transaction signature (base58).
    const tx = await program.methods
      .initialize(mockUsdtMint)
      .accounts({
        globalState:   globalStatePda,
        payer:         provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("  tx:", tx);

    // program.account.globalState.fetch(pda) — makes a `getAccountInfo`
    // RPC call to surfpool, deserializes the raw bytes using the Anchor
    // account discriminator + borsh layout defined by `GlobalState` in
    // lib.rs, and returns a typed JS object.
    const state = await program.account.globalState.fetch(globalStatePda);

    // The payer of the `init` instruction becomes `platform_admin`.
    assert.equal(
      state.platformAdmin.toBase58(),
      provider.wallet.publicKey.toBase58(),
      "platform_admin should be the payer"
    );
    // The program stored the pubkey we passed as `usdt_mint`.
    assert.equal(
      state.usdtMint.toBase58(),
      mockUsdtMint.toBase58(),
      "usdt_mint should match"
    );
    // coop_counter starts at 0 — no cooperatives exist at init time.
    // `.toNumber()` converts a BN (big-number library) to a plain JS number.
    // Solana's u64 can exceed Number.MAX_SAFE_INTEGER, so Anchor always
    // returns integer fields as BN to prevent silent precision loss.
    assert.equal(state.coopCounter.toNumber(), 0, "coop_counter starts at 0");
  });
});
