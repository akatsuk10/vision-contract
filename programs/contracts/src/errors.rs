use anchor_lang::prelude::*;

#[error_code]
pub enum ContractError {
    #[msg("Bid amount must be greater than zero")]
    ZeroBidAmount,
    #[msg("Product is not in bidding phase")]
    NotInBiddingPhase,
    #[msg("Bidding period has ended")]
    BiddingPeriodEnded,
    #[msg("Launch date has not arrived yet")]
    LaunchDateNotReached,
    #[msg("Bid has already been processed")]
    BidAlreadyProcessed,
    #[msg("Bid is not approved")]
    BidNotApproved,
    #[msg("Bid is not rejected")]
    BidNotRejected,
    #[msg("Only product owner can perform this action")]
    UnauthorizedAccess,
    #[msg("All IPO slots are filled")]
    AllSlotsFilled,
    #[msg("Invalid launch date")]
    InvalidLaunchDate,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Tokens already claimed")]
    TokensAlreadyClaimed,
    #[msg("Funds already claimed")]
    FundsAlreadyClaimed,
    #[msg("Insufficient tokens in pool")]
    InsufficientTokens,
}