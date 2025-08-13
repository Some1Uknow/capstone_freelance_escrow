use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Invalid status for this action")]
    InvalidStatus,
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Insufficient funds to deposit")]
    InsufficientFunds,
    #[msg("Invalid amount specified")]
    InvalidAmount,
    #[msg("Work link cannot be empty")]
    InvalidWorkLink,
    #[msg("Work link is too long")]
    WorkLinkTooLong,
    #[msg("Escrow is already complete")]
    EscrowAlreadyComplete,
    #[msg("Invalid timeout period (must be 1-90 days)")]
    InvalidTimeout,
}
