use anchor_lang::prelude::error_code;

#[error_code]
pub enum PumpFunError {
  #[msg("Funding Already Raised")]
  AlreadyRaised,
  #[msg("Not enough available supply")]
  NotEnoughSuppply,
  #[msg("Incorrect value of SOL sent")]
  InvalidSolAmount,
}
