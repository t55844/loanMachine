// src/services/privy/mod.rs
//
// Server-side Privy integration.
//
// Current responsibilities:
//   - Hold the public app_id (sent to browser)
//   - Produce the `window.APP_CONFIG` JS blob
//
// Future responsibilities (when you add them):
//   - Verify Privy-issued JWTs on protected endpoints
//   - Validate webhook signatures
//   - Make server-to-server calls to the Privy API

#[derive(Debug, thiserror::Error)]
pub enum PrivyError{
    #[error("PRIVY_APP_ID não configurado")]
    MissingPrivyAppId,
}

pub struct PrivyService {
    app_id:String,

}

impl PrivyService {
    pub fn from_env() -> Result<Self, PrivyError>{
        let app_id = std::env::var("PRIVY_APP_ID")
        .map_err(|_| PrivyError::MissingPrivyAppId)?;
        if app_id.trim().is_empty(){
            return Err(PrivyError::MissingPrivyAppId);
        }

        Ok(Self { app_id })
    }

    pub fn app_id(&self) -> String {
        self.app_id.clone()
    }

}