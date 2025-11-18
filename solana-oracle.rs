use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

declare_id!("GqEkgwLMtTZ2XmP4LnwJUQbAQWUR3PMfTN8pNojBH6ks");

// ============================================================================
// Constants
// ============================================================================

const MAX_PUBLISHERS: usize = 100;
const MIN_STAKE_AMOUNT: u64 = 10_000_000_000; // 10,000 tokens with 9 decimals
const STALENESS_THRESHOLD: i64 = 30;
const HALTED_THRESHOLD: i64 = 60;
const OUTLIER_MAD_MULTIPLIER: i64 = 3;
const EMA_ALPHA_SCALED: i64 = 100_000; // 0.1 * 1_000_000
const UNBONDING_PERIOD: i64 = 604_800; // 7 days
const PROGRAM_VERSION: u8 = 1;

// ============================================================================
// Error Codes
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("Price feed is not in trading status")]
    PriceNotTrading,
    #[msg("Price data is stale")]
    PriceStale,
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    #[msg("Publisher not authorized for this feed")]
    UnauthorizedPublisher,
    #[msg("Not enough publishers reporting")]
    InsufficientPublishers,
    #[msg("Invalid price data")]
    InvalidPrice,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Confidence interval too large")]
    ConfidenceTooLarge,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Publisher already exists")]
    PublisherExists,
    #[msg("Unauthorized action")]
    Unauthorized,
    #[msg("Proposal not approved")]
    ProposalNotApproved,
    #[msg("Unbonding period not elapsed")]
    UnbondingPeriodActive,
    #[msg("System is paused")]
    SystemPaused,
    #[msg("Invalid slash percentage")]
    InvalidSlashPercentage,
    #[msg("Voting period ended")]
    VotingPeriodEnded,
    #[msg("Quorum not reached")]
    QuorumNotReached,
    #[msg("Timelock not expired")]
    TimelockNotExpired,
    #[msg("Publishers array is full")]
    PublishersArrayFull,
    #[msg("Invalid proposal type")]
    InvalidProposalType,
    #[msg("Voting period active")]
    VotingPeriodActive,
}

// ============================================================================
// Enums
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PriceStatus {
    Trading,
    Halted,
    Auction,
    Unknown,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AssetType {
    Crypto,
    Equity,
    Forex,
    Commodity,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PriceType {
    Spot,
    Futures,
    Option,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VoteType {
    Yes,
    No,
    Abstain,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalType {
    UpdateRewardRate { new_rate: u64 },
    UpdateMinPublishers { feed: Pubkey, new_min: u8 },
    SlashPublisher { publisher: Pubkey, percentage: u8 },
    EmergencyPause,
    EmergencyUnpause,
    UpdateGovernanceParams { 
        proposal_threshold: Option<u64>,
        voting_period: Option<u64>,
        quorum_percentage: Option<u8>,
        timelock_duration: Option<u64>,
    },
}

// ============================================================================
// Data Structures
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PriceData {
    pub price: i64,
    pub confidence: u64,
    pub exponent: i32,
    pub timestamp: i64,
    pub slot: u64,
    pub status: PriceStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct PublisherPrice {
    pub publisher: Pubkey,
    pub price: i64,
    pub confidence: u64,
    pub timestamp: i64,
    pub slot: u64,
    pub stake: u64,
    pub active: bool, // Track if this slot is in use
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct EmaData {
    pub ema_price: i64,
    pub ema_confidence: u64,
    pub num_observations: u64,
}

// ============================================================================
// Events
// ============================================================================

#[event]
pub struct PriceUpdated {
    pub product: Pubkey,
    pub symbol: String,
    pub price: i64,
    pub confidence: u64,
    pub timestamp: i64,
    pub slot: u64,
    pub status: PriceStatus,
}

#[event]
pub struct PublisherAdded {
    pub publisher: Pubkey,
    pub authority: Pubkey,
    pub stake_amount: u64,
    pub name: String,
}

#[event]
pub struct PublisherSlashed {
    pub publisher: Pubkey,
    pub slash_amount: u64,
    pub slash_percentage: u8,
    pub reason: String,
}

#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub description: String,
}

#[event]
pub struct ProposalExecuted {
    pub proposal_id: u64,
    pub proposal_type: ProposalType,
}

#[event]
pub struct SystemPaused {
    pub timestamp: i64,
    pub authority: Pubkey,
}

#[event]
pub struct SystemUnpaused {
    pub timestamp: i64,
    pub authority: Pubkey,
}

// ============================================================================
// Accounts
// ============================================================================

#[account]
pub struct GlobalState {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub vault_authority: Pubkey,
    pub governance: Pubkey,
    pub paused: bool,
    pub total_products: u64,
    pub total_publishers: u64,
    pub version: u8,
    pub bump: u8,
    pub vault_authority_bump: u8,
}

#[account]
pub struct ProductAccount {
    pub symbol: String,
    pub asset_type: AssetType,
    pub description: String,
    pub price_account: Pubkey,
    pub authority: Pubkey,
    pub bump: u8,
}

#[account]
pub struct PriceAccount {
    pub product_account: Pubkey,
    pub price_type: PriceType,
    pub aggregate: PriceData,
    pub publishers: [PublisherPrice; MAX_PUBLISHERS], // Fixed-size array
    pub publisher_count: u8,
    pub min_publishers: u8,
    pub last_update_slot: u64,
    pub ema: EmaData,
    pub authority: Pubkey,
    pub exponent: i32,
    pub bump: u8,
}

#[account]
pub struct PublisherAccount {
    pub authority: Pubkey,
    pub staked_amount: u64,
    pub stake_account: Pubkey,
    pub reputation: u64,
    pub name: String,
    pub registered_at: i64,
    pub slash_count: u32,
    pub last_slash_slot: u64,
    pub unbonding_amount: u64,
    pub unbonding_start: i64,
    pub bump: u8,
}

#[account]
pub struct TokenVault {
    pub total_staked: u64,
    pub total_rewards_distributed: u64,
    pub reward_rate: u64,
    pub last_distribution_slot: u64,
    pub token_mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub vault_authority: Pubkey,
    pub authority: Pubkey,
    pub bump: u8,
}

#[account]
pub struct GovernanceState {
    pub governance_token: Pubkey,
    pub proposal_threshold: u64,
    pub voting_period: u64,
    pub quorum_percentage: u8,
    pub timelock_duration: u64,
    pub proposal_count: u64,
    pub total_supply: u64, // Store total supply for quorum calculation
    pub authority: Pubkey,
    pub bump: u8,
}

#[account]
pub struct Proposal {
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub start_slot: u64,
    pub end_slot: u64,
    pub executed: bool,
    pub execution_time: i64,
    pub proposal_id: u64,
    pub bump: u8,
}

// ============================================================================
// Program
// ============================================================================

#[program]
pub mod sfdn_oracle {
    use super::*;

    // ========================================================================
    // Initialization Instructions
    // ========================================================================

    pub fn initialize_program(
        ctx: Context<InitializeProgram>,
        reward_rate: u64,
        proposal_threshold: u64,
        voting_period: u64,
        quorum_percentage: u8,
        timelock_duration: u64,
        total_supply: u64,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.authority = ctx.accounts.authaority.key();
        global_state.token_mint = ctx.accounts.token_mint.key();
        global_state.token_vault = ctx.accounts.token_vault.key();
        global_state.vault_authority = ctx.accounts.vault_authority.key();
        global_state.governance = ctx.accounts.governance_state.key();
        global_state.paused = false;
        global_state.total_products = 0;
        global_state.total_publishers = 0;
        global_state.version = PROGRAM_VERSION;
        global_state.bump = ctx.bumps.global_state;
        global_state.vault_authority_bump = ctx.bumps.vault_authority;

        let token_vault = &mut ctx.accounts.token_vault;
        token_vault.total_staked = 0;
        token_vault.total_rewards_distributed = 0;
        token_vault.reward_rate = reward_rate;
        token_vault.last_distribution_slot = Clock::get()?.slot;
        token_vault.token_mint = ctx.accounts.token_mint.key();
        token_vault.vault_token_account = ctx.accounts.vault_token_account.key();
        token_vault.vault_authority = ctx.accounts.vault_authority.key();
        token_vault.authority = ctx.accounts.authority.key();
        token_vault.bump = ctx.bumps.token_vault;

        let governance = &mut ctx.accounts.governance_state;
        governance.governance_token = ctx.accounts.token_mint.key();
        governance.proposal_threshold = proposal_threshold;
        governance.voting_period = voting_period;
        governance.quorum_percentage = quorum_percentage;
        governance.timelock_duration = timelock_duration;
        governance.proposal_count = 0;
        governance.total_supply = total_supply;
        governance.authority = ctx.accounts.authority.key();
        governance.bump = ctx.bumps.governance_state;

        Ok(())
    }

    pub fn create_product(
        ctx: Context<CreateProduct>,
        symbol: String,
        asset_type: AssetType,
        description: String,
        price_type: PriceType,
        min_publishers: u8,
        exponent: i32,
    ) -> Result<()> {
        require!(!ctx.accounts.global_state.paused, ErrorCode::SystemPaused);

        let product = &mut ctx.accounts.product_account;
        product.symbol = symbol.clone();
        product.asset_type = asset_type;
        product.description = description;
        product.price_account = ctx.accounts.price_account.key();
        product.authority = ctx.accounts.authority.key();
        product.bump = ctx.bumps.product_account;

        let price_account = &mut ctx.accounts.price_account;
        price_account.product_account = ctx.accounts.product_account.key();
        price_account.price_type = price_type;
        price_account.aggregate = PriceData::default();
        price_account.publishers = [PublisherPrice::default(); MAX_PUBLISHERS];
        price_account.publisher_count = 0;
        price_account.min_publishers = min_publishers;
        price_account.last_update_slot = 0;
        price_account.ema = EmaData::default();
        price_account.authority = ctx.accounts.authority.key();
        price_account.exponent = exponent;
        price_account.bump = ctx.bumps.price_account;

        ctx.accounts.global_state.total_products += 1;

        Ok(())
    }

    pub fn add_publisher(
        ctx: Context<AddPublisher>,
        name: String,
        initial_stake: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.global_state.paused, ErrorCode::SystemPaused);
        require!(
            initial_stake >= MIN_STAKE_AMOUNT,
            ErrorCode::InsufficientStake
        );

        // Transfer stake to vault using vault authority
        let cpi_accounts = Transfer {
            from: ctx.accounts.publisher_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.publisher_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, initial_stake)?;

        let publisher = &mut ctx.accounts.publisher_account;
        publisher.authority = ctx.accounts.publisher_authority.key();
        publisher.staked_amount = initial_stake;
        publisher.stake_account = ctx.accounts.publisher_token_account.key();
        publisher.reputation = 100;
        publisher.name = name.clone();
        publisher.registered_at = Clock::get()?.unix_timestamp;
        publisher.slash_count = 0;
        publisher.last_slash_slot = 0;
        publisher.unbonding_amount = 0;
        publisher.unbonding_start = 0;
        publisher.bump = ctx.bumps.publisher_account;

        ctx.accounts.token_vault.total_staked += initial_stake;
        ctx.accounts.global_state.total_publishers += 1;

        emit!(PublisherAdded {
            publisher: ctx.accounts.publisher_account.key(),
            authority: ctx.accounts.publisher_authority.key(),
            stake_amount: initial_stake,
            name,
        });

        Ok(())
    }

    // ========================================================================
    // Publisher Instructions
    // ========================================================================

    pub fn update_price(
        ctx: Context<UpdatePrice>,
        price: i64,
        confidence: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.global_state.paused, ErrorCode::SystemPaused);
        require!(price > 0, ErrorCode::InvalidPrice);
        
        let clock = Clock::get()?;
        let timestamp = clock.unix_timestamp;
        let slot = clock.slot;

        require!(timestamp > 0, ErrorCode::InvalidTimestamp);

        let price_account = &mut ctx.accounts.price_account;
        let publisher = &ctx.accounts.publisher_account;

        let publisher_price = PublisherPrice {
            publisher: publisher.authority,
            price,
            confidence,
            timestamp,
            slot,
            stake: publisher.staked_amount,
            active: true,
        };

        // Find existing slot or add new one
        let mut found = false;
        for i in 0..MAX_PUBLISHERS {
            if price_account.publishers[i].active && 
               price_account.publishers[i].publisher == publisher.authority {
                price_account.publishers[i] = publisher_price;
                found = true;
                break;
            }
        }

        if !found {
            // Find empty slot
            let mut added = false;
            for i in 0..MAX_PUBLISHERS {
                if !price_account.publishers[i].active {
                    price_account.publishers[i] = publisher_price;
                    price_account.publisher_count += 1;
                    added = true;
                    break;
                }
            }
            require!(added, ErrorCode::PublishersArrayFull);
        }

        price_account.last_update_slot = slot;

        // Trigger aggregation if enough publishers
        if price_account.publisher_count >= price_account.min_publishers {
            aggregate_prices_internal(price_account, &ctx.accounts.product_account.symbol)?;
        }

        Ok(())
    }

    pub fn stake_tokens(
        ctx: Context<StakeTokens>,
        amount: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.global_state.paused, ErrorCode::SystemPaused);
        require!(amount > 0, ErrorCode::InsufficientStake);

        let cpi_accounts = Transfer {
            from: ctx.accounts.publisher_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.publisher_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        ctx.accounts.publisher_account.staked_amount += amount;
        ctx.accounts.token_vault.total_staked += amount;

        Ok(())
    }

    pub fn unstake_tokens(
        ctx: Context<UnstakeTokens>,
        amount: u64,
    ) -> Result<()> {
        require!(!ctx.accounts.global_state.paused, ErrorCode::SystemPaused);
        
        let publisher = &mut ctx.accounts.publisher_account;
        let remaining = publisher.staked_amount.checked_sub(amount)
            .ok_or(ErrorCode::InsufficientStake)?;
        
        require!(remaining >= MIN_STAKE_AMOUNT, ErrorCode::InsufficientStake);

        publisher.unbonding_amount = amount;
        publisher.unbonding_start = Clock::get()?.unix_timestamp;
        publisher.staked_amount = remaining;

        Ok(())
    }

    pub fn withdraw_unbonded(
        ctx: Context<WithdrawUnbonded>,
    ) -> Result<()> {
        let publisher = &mut ctx.accounts.publisher_account;
        let clock = Clock::get()?;
        
        require!(
            clock.unix_timestamp - publisher.unbonding_start >= UNBONDING_PERIOD,
            ErrorCode::UnbondingPeriodActive
        );

        let amount = publisher.unbonding_amount;
        require!(amount > 0, ErrorCode::InsufficientStake);

        // Transfer using vault authority PDA
        let vault_authority_bump = ctx.accounts.global_state.vault_authority_bump;
        let seeds = &[
            b"vault_authority".as_ref(),
            &[vault_authority_bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.publisher_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        publisher.unbonding_amount = 0;
        publisher.unbonding_start = 0;
        ctx.accounts.token_vault.total_staked = ctx.accounts.token_vault.total_staked
            .checked_sub(amount)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

    // ========================================================================
    // Aggregation
    // ========================================================================

    pub fn aggregate_price(
        ctx: Context<AggregatePrice>,
    ) -> Result<()> {
        aggregate_prices_internal(
            &mut ctx.accounts.price_account,
            &ctx.accounts.product_account.symbol
        )?;
        Ok(())
    }

    // ========================================================================
    // Governance Instructions
    // ========================================================================

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_type: ProposalType,
        description: String,
    ) -> Result<()> {
        require!(!ctx.accounts.global_state.paused, ErrorCode::SystemPaused);
        
        let proposer_balance = ctx.accounts.proposer_token_account.amount;
        require!(
            proposer_balance >= ctx.accounts.governance_state.proposal_threshold,
            ErrorCode::Unauthorized
        );

        let clock = Clock::get()?;
        let proposal = &mut ctx.accounts.proposal;
        let governance = &mut ctx.accounts.governance_state;

        proposal.proposer = ctx.accounts.proposer.key();
        proposal.proposal_type = proposal_type.clone();
        proposal.description = description.clone();
        proposal.yes_votes = 0;
        proposal.no_votes = 0;
        proposal.abstain_votes = 0;
        proposal.start_slot = clock.slot;
        proposal.end_slot = clock.slot + governance.voting_period;
        proposal.executed = false;
        proposal.execution_time = 0;
        proposal.proposal_id = governance.proposal_count;
        proposal.bump = ctx.bumps.proposal;

        governance.proposal_count += 1;

        emit!(ProposalCreated {
            proposal_id: proposal.proposal_id,
            proposer: proposal.proposer,
            proposal_type,
            description,
        });

        Ok(())
    }

    pub fn vote_proposal(
        ctx: Context<VoteProposal>,
        vote: VoteType,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let proposal = &mut ctx.accounts.proposal;

        require!(clock.slot <= proposal.end_slot, ErrorCode::VotingPeriodEnded);

        let vote_weight = ctx.accounts.voter_token_account.amount;

        match vote {
            VoteType::Yes => proposal.yes_votes += vote_weight,
            VoteType::No => proposal.no_votes += vote_weight,
            VoteType::Abstain => proposal.abstain_votes += vote_weight,
        }

        Ok(())
    }

    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let proposal = &mut ctx.accounts.proposal;
        let governance = &ctx.accounts.governance_state;

        require!(clock.slot > proposal.end_slot, ErrorCode::VotingPeriodActive);
        require!(!proposal.executed, ErrorCode::ProposalNotApproved);

        // Check quorum
        let total_votes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
        let quorum = (governance.total_supply as u128 * governance.quorum_percentage as u128) / 100;
        
        require!(total_votes as u128 >= quorum, ErrorCode::QuorumNotReached);
        require!(proposal.yes_votes > proposal.no_votes, ErrorCode::ProposalNotApproved);

        // Timelock mechanism
        if proposal.execution_time == 0 {
            proposal.execution_time = clock.unix_timestamp + governance.timelock_duration as i64;
            return Ok(());
        }

        require!(
            clock.unix_timestamp >= proposal.execution_time,
            ErrorCode::TimelockNotExpired
        );

        proposal.executed = true;

        emit!(ProposalExecuted {
            proposal_id: proposal.proposal_id,
            proposal_type: proposal.proposal_type.clone(),
        });

        Ok(())
    }

    pub fn execute_governance_action(
        ctx: Context<ExecuteGovernanceAction>,
    ) -> Result<()> {
        let proposal = &ctx.accounts.proposal;
        require!(proposal.executed, ErrorCode::ProposalNotApproved);

        match &proposal.proposal_type {
            ProposalType::UpdateRewardRate { new_rate } => {
                ctx.accounts.token_vault.reward_rate = *new_rate;
            },
            ProposalType::UpdateMinPublishers { feed: _, new_min } => {
                if let Some(price_account) = ctx.accounts.price_account.as_mut() {
                    price_account.min_publishers = *new_min;
                }
            },
            ProposalType::EmergencyPause => {
                ctx.accounts.global_state.paused = true;
                emit!(SystemPaused {
                    timestamp: Clock::get()?.unix_timestamp,
                    authority: ctx.accounts.authority.key(),
                });
            },
            ProposalType::EmergencyUnpause => {
                ctx.accounts.global_state.paused = false;
                emit!(SystemUnpaused {
                    timestamp: Clock::get()?.unix_timestamp,
                    authority: ctx.accounts.authority.key(),
                });
            },
            ProposalType::UpdateGovernanceParams { 
                proposal_threshold,
                voting_period,
                quorum_percentage,
                timelock_duration,
            } => {
                let gov = &mut ctx.accounts.governance_state;
                if let Some(threshold) = proposal_threshold {
                    gov.proposal_threshold = *threshold;
                }
                if let Some(period) = voting_period {
                    gov.voting_period = *period;
                }
                if let Some(quorum) = quorum_percentage {
                    gov.quorum_percentage = *quorum;
                }
                if let Some(timelock) = timelock_duration {
                    gov.timelock_duration = *timelock;
                }
            },
            ProposalType::SlashPublisher { publisher: _, percentage } => {
                if let Some(pub_account) = ctx.accounts.publisher_account.as_mut() {
                    let slash_amount = (pub_account.staked_amount as u128 * *percentage as u128) / 100;
                    let slash_amount = slash_amount as u64;

                    pub_account.staked_amount = pub_account.staked_amount
                        .checked_sub(slash_amount)
                        .ok_or(ErrorCode::Overflow)?;
                    pub_account.slash_count += 1;
                    pub_account.last_slash_slot = Clock::get()?.slot;

                    ctx.accounts.token_vault.total_staked = ctx.accounts.token_vault.total_staked
                        .checked_sub(slash_amount)
                        .ok_or(ErrorCode::Overflow)?;

                    emit!(PublisherSlashed {
                        publisher: pub_account.key(),
                        slash_amount,
                        slash_percentage: *percentage,
                        reason: "Governance proposal".to_string(),
                    });
                }
            },
        }

        Ok(())
    }

    pub fn emergency_pause(
        ctx: Context<EmergencyPause>,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.global_state.authority,
            ErrorCode::Unauthorized
        );

        ctx.accounts.global_state.paused = true;

        emit!(SystemPaused {
            timestamp: Clock::get()?.unix_timestamp,
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    pub fn emergency_unpause(
        ctx: Context<EmergencyUnpause>,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.global_state.authority,
            ErrorCode::Unauthorized
        );

        ctx.accounts.global_state.paused = false;

        emit!(SystemUnpaused {
            timestamp: Clock::get()?.unix_timestamp,
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }
}

// ============================================================================
// Internal Functions (Optimized)
// ============================================================================

fn aggregate_prices_internal(price_account: &mut PriceAccount, symbol: &str) -> Result<()> {
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    // Collect active, non-stale prices
    let mut valid_prices: Vec<&PublisherPrice> = price_account
        .publishers
        .iter()
        .filter(|p| p.active && current_time - p.timestamp < STALENESS_THRESHOLD)
        .collect();

    if valid_prices.is_empty() {
        price_account.aggregate.status = PriceStatus::Unknown;
        return Ok(());
    }

    // Sort by price (in-place, no cloning)
    valid_prices.sort_by_key(|p| p.price);

    // Remove outliers using MAD
    let filtered_prices = filter_outliers_optimized(&valid_prices);

    if filtered_prices.len() < price_account.min_publishers as usize {
        price_account.aggregate.status = PriceStatus::Unknown;
        return Ok(());
    }

    // Calculate stake-weighted median
    let median_price = calculate_stake_weighted_median_optimized(&filtered_prices)?;

    // Calculate confidence (using u128 to prevent overflow)
    let confidence = calculate_confidence_safe(&filtered_prices, median_price)?;

    // Determine status
    let status = determine_status_optimized(&valid_prices, price_account.min_publishers, current_time);

    // Update aggregate
    price_account.aggregate = PriceData {
        price: median_price,
        confidence,
        exponent: price_account.exponent,
        timestamp: current_time,
        slot: clock.slot,
        status: status.clone(),
    };

    // Update EMA
    price_account.ema = update_ema(&price_account.ema, median_price, confidence);

    emit!(PriceUpdated {
        product: price_account.product_account,
        symbol: symbol.to_string(),
        price: median_price,
        confidence,
        timestamp: current_time,
        slot: clock.slot,
        status,
    });

    Ok(())
}

fn filter_outliers_optimized<'a>(prices: &[&'a PublisherPrice]) -> Vec<&'a PublisherPrice> {
    if prices.len() < 3 {
        return prices.to_vec();
    }

    let median_idx = prices.len() / 2;
    let median = prices[median_idx].price;

    // Calculate MAD without additional allocation
    let mut deviations: Vec<i64> = prices
        .iter()
        .map(|p| (p.price - median).abs())
        .collect();
    deviations.sort_unstable();

    let mad = deviations[deviations.len() / 2];
    let threshold = mad.saturating_mul(OUTLIER_MAD_MULTIPLIER);

    prices
        .iter()
        .filter(|p| (p.price - median).abs() <= threshold)
        .copied()
        .collect()
}

fn calculate_stake_weighted_median_optimized(prices: &[&PublisherPrice]) -> Result<i64> {
    let total_stake: u128 = prices.iter().map(|p| p.stake as u128).sum();
    let median_stake = total_stake / 2;

    let mut cumulative_stake: u128 = 0;
    for price in prices {
        cumulative_stake += price.stake as u128;
        if cumulative_stake >= median_stake {
            return Ok(price.price);
        }
    }

    Ok(prices[0].price)
}

fn calculate_confidence_safe(prices: &[&PublisherPrice], median: i64) -> Result<u64> {
    let total_stake: u128 = prices.iter().map(|p| p.stake as u128).sum();
    
    if total_stake == 0 {
        return Ok(1);
    }

    let variance: u128 = prices
        .iter()
        .map(|p| {
            let diff = (p.price - median).abs() as i128;
            let diff_squared = (diff * diff) as u128;
            (diff_squared * p.stake as u128) / total_stake
        })
        .sum();

    let std_dev = (variance as f64).sqrt() as u64;
    Ok(std_dev.max(1))
}

fn determine_status_optimized(
    prices: &[&PublisherPrice],
    min_publishers: u8,
    current_time: i64
) -> PriceStatus {
    if prices.len() < min_publishers as usize {
        return PriceStatus::Unknown;
    }

    if let Some(latest) = prices.iter().map(|p| p.timestamp).max() {
        if current_time - latest > HALTED_THRESHOLD {
            return PriceStatus::Halted;
        }
    }

    PriceStatus::Trading
}

fn update_ema(current_ema: &EmaData, new_price: i64, new_confidence: u64) -> EmaData {
    if current_ema.num_observations == 0 {
        return EmaData {
            ema_price: new_price,
            ema_confidence: new_confidence,
            num_observations: 1,
        };
    }

    let one_minus_alpha = 1_000_000 - EMA_ALPHA_SCALED;

    let new_ema_price = ((EMA_ALPHA_SCALED as i128 * new_price as i128 
        + one_minus_alpha as i128 * current_ema.ema_price as i128) / 1_000_000) as i64;
    
    let new_ema_confidence = ((EMA_ALPHA_SCALED as u128 * new_confidence as u128 
        + one_minus_alpha as u128 * current_ema.ema_confidence as u128) / 1_000_000) as u64;

    EmaData {
        ema_price: new_ema_price,
        ema_confidence: new_ema_confidence,
        num_observations: current_ema.num_observations.saturating_add(1),
    }
}

// ============================================================================
// Context Structs
// ============================================================================

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 1 + 8 + 8 + 1 + 1 + 1,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    /// CHECK: PDA used as vault authority for token transfers
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 8 + 8 + 8 + 32 + 32 + 32 + 32 + 1,
        seeds = [b"token_vault"],
        bump
    )]
    pub token_vault: Account<'info, TokenVault>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8 + 1 + 8 + 8 + 8 + 32 + 1,
        seeds = [b"governance"],
        bump
    )]
    pub governance_state: Account<'info, GovernanceState>,

    pub token_mint: Account<'info, Mint>,
    
    #[account(
        constraint = vault_token_account.mint == token_mint.key(),
        constraint = vault_token_account.owner == vault_authority.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct CreateProduct<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        payer = authority,
        space = 8 + 64 + 1 + 256 + 32 + 32 + 1,
        seeds = [b"product", symbol.as_bytes()],
        bump
    )]
    pub product_account: Account<'info, ProductAccount>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 128 + (MAX_PUBLISHERS * 96) + 1 + 1 + 8 + 32 + 32 + 4 + 1,
        seeds = [b"price", symbol.as_bytes()],
        bump
    )]
    pub price_account: Account<'info, PriceAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddPublisher<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 32 + 8 + 64 + 8 + 4 + 8 + 8 + 8 + 1,
        seeds = [b"publisher", publisher_authority.key().as_ref()],
        bump
    )]
    pub publisher_account: Account<'info, PublisherAccount>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenVault>,

    #[account(
        mut,
        constraint = publisher_token_account.mint == token_vault.token_mint,
        constraint = publisher_token_account.owner == publisher_authority.key()
    )]
    pub publisher_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == token_vault.vault_token_account
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub publisher_authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    pub global_state: Account<'info, GlobalState>,

    pub product_account: Account<'info, ProductAccount>,

    #[account(
        mut,
        seeds = [b"price", product_account.symbol.as_bytes()],
        bump = price_account.bump
    )]
    pub price_account: Account<'info, PriceAccount>,

    #[account(
        seeds = [b"publisher", publisher_authority.key().as_ref()],
        bump = publisher_account.bump,
        constraint = publisher_account.authority == publisher_authority.key()
    )]
    pub publisher_account: Account<'info, PublisherAccount>,

    pub publisher_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"publisher", publisher_authority.key().as_ref()],
        bump = publisher_account.bump
    )]
    pub publisher_account: Account<'info, PublisherAccount>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenVault>,

    #[account(
        mut,
        constraint = publisher_token_account.owner == publisher_authority.key()
    )]
    pub publisher_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == token_vault.vault_token_account
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub publisher_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"publisher", publisher_authority.key().as_ref()],
        bump = publisher_account.bump
    )]
    pub publisher_account: Account<'info, PublisherAccount>,

    pub publisher_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawUnbonded<'info> {
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"publisher", publisher_authority.key().as_ref()],
        bump = publisher_account.bump
    )]
    pub publisher_account: Account<'info, PublisherAccount>,

    /// CHECK: PDA vault authority
    #[account(
        seeds = [b"vault_authority"],
        bump = global_state.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"token_vault"],
        bump = token_vault.bump
    )]
    pub token_vault: Account<'info, TokenVault>,

    #[account(
        mut,
        constraint = publisher_token_account.owner == publisher_authority.key()
    )]
    pub publisher_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == token_vault.vault_token_account
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub publisher_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AggregatePrice<'info> {
    pub product_account: Account<'info, ProductAccount>,
    
    #[account(mut)]
    pub price_account: Account<'info, PriceAccount>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub governance_state: Account<'info, GovernanceState>,

    #[account(
        init,
        payer = proposer,
        space = 8 + 32 + 256 + 256 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 8 + 1,
        seeds = [b"proposal", governance_state.proposal_count.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        constraint = proposer_token_account.owner == proposer.key()
    )]
    pub proposer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        constraint = voter_token_account.owner == voter.key()
    )]
    pub voter_token_account: Account<'info, TokenAccount>,

    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    pub governance_state: Account<'info, GovernanceState>,
}

#[derive(Accounts)]
pub struct ExecuteGovernanceAction<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,

    pub proposal: Account<'info, Proposal>,

    pub governance_state: Account<'info, GovernanceState>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenVault>,

    #[account(mut)]
    pub price_account: Option<Account<'info, PriceAccount>>,

    #[account(mut)]
    pub publisher_account: Option<Account<'info, PublisherAccount>>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(constraint = authority.key() == global_state.authority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct EmergencyUnpause<'info> {
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(constraint = authority.key() == global_state.authority)]
    pub authority: Signer<'info>,
}

// ============================================================================
// Default Implementations
// ============================================================================

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Unknown
    }
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Crypto
    }
}

impl Default for PriceType {
    fn default() -> Self {
        PriceType::Spot
    }
}