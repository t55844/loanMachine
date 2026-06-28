use anchor_lang::prelude::*;

declare_id!("2VzdQShLpznATZLPz8XTsMZCP1KNaohyTX77xDdJzanE");

#[program]
pub mod loan_machine {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, usdt_mint: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        state.platform_admin = ctx.accounts.payer.key();
        state.usdt_mint      = usdt_mint;
        state.coop_counter   = 0;
        state.bump           = ctx.bumps.global_state;
        Ok(())
    }
}

// ─── State ───────────────────────────────────────────────────────────────────

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub platform_admin: Pubkey, // 32
    pub usdt_mint:      Pubkey, // 32
    pub coop_counter:   u64,    //  8
    pub bump:           u8,     //  1
}

// ─── Accounts ────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer  = payer,
        space  = 8 + GlobalState::INIT_SPACE,
        seeds  = [b"global"],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
