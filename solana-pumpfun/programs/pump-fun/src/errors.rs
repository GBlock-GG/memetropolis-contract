use anchor_lang::prelude::error_code;

#[error_code]
pub enum PumpFunError {
  #[msg("Already Initialized")]
  AlreadyInitialized,
  #[msg("Funding Already Raised")]
  AlreadyRaised,
  #[msg("Not enough available supply")]
  NotEnoughSuppply,
  #[msg("Incorrect value of SOL sent")]
  InvalidSolAmount,
  #[msg("BondingCurve: Input must be greater than zero")]
  InvalidInput,
  #[msg("Slippage Exceed")]
  SlippageExceed
}

#[error_code]
pub enum OftError {
    Unauthorized,
    InvalidSender,
    InvalidDecimals,
    SlippageExceeded,
    InvalidTokenMint,
    InvalidReceiver,
    InvalidTokenEscrow,
    InvalidTokenDest,
    InvalidOptions,
    InvalidEndpointProgram,
    RateLimitExceeded,
}