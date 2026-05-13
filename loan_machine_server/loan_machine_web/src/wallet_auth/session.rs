use leptos::prelude::*;
use loan_machine_models::wallet_address::WalletAddress;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WalletSession  {
    /// Initial paint and during restore. UI should show a neutral skeleton.
    Restoring,
    /// Restore finished, no session.
    Disconnected,
    /// Live session.
    Connected { wallet: WalletAddress },
}

impl WalletSession {
    pub fn wallet(&self) -> Option<&WalletAddress> {
        if let Self::Connected { wallet } = self { Some(wallet) } else { None }
    }
}

#[derive(Copy, Clone)]
pub struct WalletCtx {
    pub session: ReadSignal<WalletSession>,
    pub set:     WriteSignal<WalletSession>,
}

pub fn provide_wallet() -> WalletCtx {
    let (session, set) = signal(WalletSession::Restoring);
    let ctx = WalletCtx { session, set };
    provide_context(ctx);
    ctx
}

pub fn use_wallet() -> WalletCtx {
    expect_context::<WalletCtx>()
}