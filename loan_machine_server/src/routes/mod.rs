// routes/mod.rs
mod donate;
mod vinculate_member_to_wallet;

pub use donate::prepare_donation;
pub use vinculate_member_to_wallet::prepare_vinculation_to_wallet;
