use crate::{constants::*, errors::DaoError};
use anchor_lang::prelude::*;

#[account]
pub struct DaoSetup {
    // Unique identifier for this DAO configuration, used in PDA derivation
    pub seed: u64,

    // Price at which tokens are issued or sold within the DAO
    pub issue_price: u64,

    // Quantity of tokens issued in each transaction or offering
    pub issue_amount: u64,

    // Fee required to submit a proposal to the DAO
    pub proposal_fee: u64,

    // Maximum number of tokens that can be issued for this DAO
    pub max_supply: u64,

    // Minimum number of votes required for a proposal to be considered valid
    pub min_quorum: u64,

    // Maximum duration a proposal can remain active for before expiring
    pub max_expiry: u64,

    // Counter keeping track of the total number of proposals submitted
    pub proposal_count: u64,

    // Bump seed for the authority PDA
    pub auth_bump: u8,

    // Bump seed for this configuration account's PDA
    pub config_bump: u8,

    // Bump seed for the token mint PDA
    pub mint_bump: u8,
    
    // Bump seed for the treasury account PDA
    pub treasury_bump: u8
}

impl DaoSetup {
    /// Total size of the account in bytes
    pub const LEN: usize = 8 + 6 * U64_L + 4 * U8_L;

    /// Initializes a new DaoConfig instance with the provided parameters
    ///
    /// # Arguments
    ///
    /// * `seed` - Unique identifier for this DAO configuration
    /// * `issue_price` - Price at which tokens are issued
    /// * `issue_amount` - Quantity of tokens issued per transaction
    /// * `proposal_fee` - Fee for submitting a proposal
    /// * `max_supply` - Maximum token supply for the DAO
    /// * `min_quorum` - Minimum votes required for a valid proposal
    /// * `max_expiry` - Maximum duration for an active proposal
    /// * `auth_bump` - Bump seed for authority PDA
    /// * `config_bump` - Bump seed for configuration PDA
    /// * `mint_bump` - Bump seed for token mint PDA
    /// * `treasury_bump` - Bump seed for treasury PDA
    pub fn init(
        &mut self,
        seed: u64,
        issue_price: u64,
        issue_amount: u64,
        proposal_fee: u64,
        max_supply: u64,
        min_quorum: u64,
        max_expiry: u64,
        auth_bump: u8,
        config_bump: u8,
        mint_bump: u8,
        treasury_bump: u8        
    ) -> Result<()> {
        self.seed = seed;
        self.issue_price = issue_price;
        self.issue_amount = issue_amount;
        self.proposal_fee = proposal_fee;
        self.max_supply = max_supply;
        self.min_quorum = min_quorum;
        self.max_expiry = max_expiry;
        self.proposal_count = 0;
        self.auth_bump = auth_bump;
        self.config_bump = config_bump;
        self.mint_bump = mint_bump;
        self.treasury_bump = treasury_bump;
        Ok(())
    }

    /// Increments the proposal count and verifies the new proposal's ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the new proposal to be added
    ///
    /// # Errors
    ///
    /// Returns an error if the proposal count overflows or if the ID doesn't match the new count
    pub fn add_proposal(&mut self, id: u64) -> Result<()> {
        self.proposal_count = self.proposal_count.checked_add(1).ok_or(DaoError::Overflow)?;
        require!(self.proposal_count == id, DaoError::InvalidProposalSeed);
        Ok(())
    }

    /// Checks if the given quorum meets the minimum requirement
    ///
    /// # Arguments
    ///
    /// * `quorum` - The quorum to be checked
    ///
    /// # Errors
    ///
    /// Returns an error if the quorum is less than the minimum required
    pub fn check_min_quorum(&self, quorum: u64) -> Result<()> {
        require!(self.min_quorum <= quorum, DaoError::InvalidQuorum);
        Ok(())
    }

    /// Verifies that the given expiry time doesn't exceed the maximum allowed
    ///
    /// # Arguments
    ///
    /// * `expiry` - The expiry time to be checked
    ///
    /// # Errors
    ///
    /// Returns an error if the expiry time is greater than the maximum allowed
    pub fn check_max_expiry(&self, expiry: u64) -> Result<()> {
        require!(self.max_expiry >= expiry, DaoError::InvalidExpiry);
        Ok(())
    }
}