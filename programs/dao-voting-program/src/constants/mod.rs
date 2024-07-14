pub const PUBKEY_LENGTH: usize = 32;
pub const ENUM_LENGTH: usize = 1;
pub const U8_LENGTH: usize = 1;
pub const U16_LENGTH: usize = 2;
pub const U32_LENGTH: usize = 4;
pub const U64_LENGTH: usize = 8;
pub const U128_LENGTH: usize = 16;
pub const BOOL_LENGTH: usize = 1;
//pub const OPTION_LENGTH: usize = 1;

// Base voting points
pub const BASE_VOTE_POINTS: u64 = 10;

// Bonus voting points for successful proposals
pub const BONUS_VOTE_POINTS: u64 = 5;

// Points for creating a proposal
pub const PROPOSAL_CREATION_POINTS: u64 = 50;

// Bonus points for a successful proposal
pub const PROPOSAL_SUCCESS_POINTS: u64 = 100;

// Reputation increase for voting
pub const VOTE_REPUTATION_INCREASE: i64 = 1;

//Reputation decrease for removing a vote
pub const VOTE_REPUTATION_DECREASE: i64 = -2;

// Reputation increase for creating a proposal
pub const PROPOSAL_CREATION_REPUTATION_INCREASE: i64 = 5;

// Reputation increase for a successful proposal
pub const PROPOSAL_SUCCESS_REPUTATION_INCREASE: i64 = 20;

// Minimum reputation required to create a proposal
pub const MIN_REPUTATION_FOR_PROPOSAL: u64 = 100;

// Maximum reputation score
pub const MAX_REPUTATION_SCORE: u64 = 10000;

// Decay factor for reputation (e.g., 5% decay per month)
pub const REPUTATION_DECAY_FACTOR: f64 = 0.95;

// Interval for reputation decay (in seconds, e.g., 30 days)
pub const REPUTATION_DECAY_INTERVAL: i64 = 30 * 24 * 60 * 60;
