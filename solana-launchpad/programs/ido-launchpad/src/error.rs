use anchor_lang::prelude::*;

#[error_code]
pub enum ErrCode {
    #[msg("Invalid start or end time")]
    InvalidTime,
    #[msg("Invalid token price")]
    InvalidPrice,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Sale is not active")]
    InvalidSaleActive,
    #[msg("Sale has ended yet")]
    InvalidSaleEnd,
    #[msg("Tokens already claimed")]
    InvalidClaim,
    #[msg("RemainedNotClaim")]
    RemainedNotClaim,
}
