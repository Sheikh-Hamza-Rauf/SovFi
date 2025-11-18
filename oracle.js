// server.js
const express = require('express');
const { Connection, PublicKey, Keypair, Transaction, SystemProgram } = require('@solana/web3.js');
const { Program, AnchorProvider, web3, BN, utils } = require('@coral-xyz/anchor');
const { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } = require('@solana/spl-token');
const IDL = require('./idl.json'); // You'll need to generate this from your program

const app = express();
app.use(express.json());

// Configuration
const PROGRAM_ID = new PublicKey('GqEkgwLMtTZ2XmP4LnwJUQbAQWUR3PMfTN8pNojBH6ks');
const RPC_URL = process.env.RPC_URL || 'https://api.devnet.solana.com';
const PORT = process.env.PORT || 3000;

// Initialize connection
const connection = new Connection(RPC_URL, 'confirmed');

// Utility function to get provider
const getProvider = (wallet) => {
  return new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
};

// Utility function to get program
const getProgram = (provider) => {
  return new Program(IDL, PROGRAM_ID, provider);
};

// ============================================================================
// HEALTH CHECK ENDPOINT
// ============================================================================

/**
 * GET /api/health
 * Health check endpoint
 */
app.get('/api/health', async (req, res) => {
  try {
    const currentSlot = await connection.getSlot();
    res.json({
      success: true,
      status: 'healthy',
      timestamp: new Date().toISOString(),
      rpcUrl: RPC_URL,
      programId: PROGRAM_ID.toBase58(),
      currentSlot: currentSlot,
      version: '1.0.0'
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      status: 'unhealthy',
      error: error.message
    });
  }
});

// ============================================================================
// INITIALIZATION ENDPOINTS
// ============================================================================

/**
 * POST /api/initialize
 * Initialize the oracle program
 */
app.post('/api/initialize', async (req, res) => {
  try {
    const {
      authoritySecretKey,
      tokenMintAddress,
      rewardRate,
      proposalThreshold,
      votingPeriod,
      quorumPercentage,
      timelockDuration,
      totalSupply
    } = req.body;

    const authority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(authoritySecretKey, 'base64'))
    );
    const tokenMint = new PublicKey(tokenMintAddress);

    const provider = getProvider({ publicKey: authority.publicKey, signTransaction: async (tx) => tx, signAllTransactions: async (txs) => txs });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault_authority')],
      program.programId
    );

    const [tokenVault] = PublicKey.findProgramAddressSync(
      [Buffer.from('token_vault')],
      program.programId
    );

    const [governanceState] = PublicKey.findProgramAddressSync(
      [Buffer.from('governance')],
      program.programId
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      vaultAuthority,
      true
    );

    const tx = await program.methods
      .initializeProgram(
        new BN(rewardRate),
        new BN(proposalThreshold),
        new BN(votingPeriod),
        quorumPercentage,
        new BN(timelockDuration),
        new BN(totalSupply)
      )
      .accounts({
        globalState,
        vaultAuthority,
        tokenVault,
        governanceState,
        tokenMint,
        vaultTokenAccount,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      accounts: {
        globalState: globalState.toBase58(),
        vaultAuthority: vaultAuthority.toBase58(),
        tokenVault: tokenVault.toBase58(),
        governanceState: governanceState.toBase58()
      }
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/products/create
 * Create a new product feed
 */
app.post('/api/products/create', async (req, res) => {
  try {
    const {
      authoritySecretKey,
      symbol,
      assetType,
      description,
      priceType,
      minPublishers,
      exponent
    } = req.body;

    const authority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(authoritySecretKey, 'base64'))
    );

    const provider = getProvider({ publicKey: authority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [productAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('product'), Buffer.from(symbol)],
      program.programId
    );

    const [priceAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('price'), Buffer.from(symbol)],
      program.programId
    );

    const assetTypeEnum = { [assetType.toLowerCase()]: {} };
    const priceTypeEnum = { [priceType.toLowerCase()]: {} };

    const tx = await program.methods
      .createProduct(
        symbol,
        assetTypeEnum,
        description,
        priceTypeEnum,
        minPublishers,
        exponent
      )
      .accounts({
        globalState,
        productAccount,
        priceAccount,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      productAccount: productAccount.toBase58(),
      priceAccount: priceAccount.toBase58()
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

// ============================================================================
// PUBLISHER ENDPOINTS
// ============================================================================

/**
 * POST /api/publishers/add
 * Add a new publisher
 */
app.post('/api/publishers/add', async (req, res) => {
  try {
    const {
      payerSecretKey,
      publisherAuthoritySecretKey,
      name,
      initialStake,
      tokenMintAddress
    } = req.body;

    const payer = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(payerSecretKey, 'base64'))
    );
    const publisherAuthority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(publisherAuthoritySecretKey, 'base64'))
    );
    const tokenMint = new PublicKey(tokenMintAddress);

    const provider = getProvider({ publicKey: payer.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [publisherAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('publisher'), publisherAuthority.publicKey.toBuffer()],
      program.programId
    );

    const [tokenVault] = PublicKey.findProgramAddressSync(
      [Buffer.from('token_vault')],
      program.programId
    );

    const [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault_authority')],
      program.programId
    );

    const publisherTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      publisherAuthority.publicKey
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      vaultAuthority,
      true
    );

    const tx = await program.methods
      .addPublisher(name, new BN(initialStake))
      .accounts({
        globalState,
        publisherAccount,
        tokenVault,
        publisherTokenAccount,
        vaultTokenAccount,
        publisherAuthority: publisherAuthority.publicKey,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer, publisherAuthority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      publisherAccount: publisherAccount.toBase58()
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/prices/update
 * Update price for a product
 */
app.post('/api/prices/update', async (req, res) => {
  try {
    const {
      publisherAuthoritySecretKey,
      symbol,
      price,
      confidence
    } = req.body;

    const publisherAuthority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(publisherAuthoritySecretKey, 'base64'))
    );

    const provider = getProvider({ publicKey: publisherAuthority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [productAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('product'), Buffer.from(symbol)],
      program.programId
    );

    const [priceAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('price'), Buffer.from(symbol)],
      program.programId
    );

    const [publisherAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('publisher'), publisherAuthority.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .updatePrice(new BN(price), new BN(confidence))
      .accounts({
        globalState,
        productAccount,
        priceAccount,
        publisherAccount,
        publisherAuthority: publisherAuthority.publicKey,
      })
      .signers([publisherAuthority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      price,
      confidence,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/publishers/stake
 * Stake additional tokens
 */
app.post('/api/publishers/stake', async (req, res) => {
  try {
    const {
      publisherAuthoritySecretKey,
      amount,
      tokenMintAddress
    } = req.body;

    const publisherAuthority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(publisherAuthoritySecretKey, 'base64'))
    );
    const tokenMint = new PublicKey(tokenMintAddress);

    const provider = getProvider({ publicKey: publisherAuthority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [publisherAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('publisher'), publisherAuthority.publicKey.toBuffer()],
      program.programId
    );

    const [tokenVault] = PublicKey.findProgramAddressSync(
      [Buffer.from('token_vault')],
      program.programId
    );

    const [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault_authority')],
      program.programId
    );

    const publisherTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      publisherAuthority.publicKey
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      vaultAuthority,
      true
    );

    const tx = await program.methods
      .stakeTokens(new BN(amount))
      .accounts({
        globalState,
        publisherAccount,
        tokenVault,
        publisherTokenAccount,
        vaultTokenAccount,
        publisherAuthority: publisherAuthority.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([publisherAuthority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      amount
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/publishers/unstake
 * Initiate unstaking tokens
 */
app.post('/api/publishers/unstake', async (req, res) => {
  try {
    const {
      publisherAuthoritySecretKey,
      amount
    } = req.body;

    const publisherAuthority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(publisherAuthoritySecretKey, 'base64'))
    );

    const provider = getProvider({ publicKey: publisherAuthority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [publisherAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('publisher'), publisherAuthority.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .unstakeTokens(new BN(amount))
      .accounts({
        globalState,
        publisherAccount,
        publisherAuthority: publisherAuthority.publicKey,
      })
      .signers([publisherAuthority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      amount,
      unbondingPeriod: '7 days'
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/publishers/withdraw-unbonded
 * Withdraw unbonded tokens after unbonding period
 */
app.post('/api/publishers/withdraw-unbonded', async (req, res) => {
  try {
    const {
      publisherAuthoritySecretKey,
      tokenMintAddress
    } = req.body;

    const publisherAuthority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(publisherAuthoritySecretKey, 'base64'))
    );
    const tokenMint = new PublicKey(tokenMintAddress);

    const provider = getProvider({ publicKey: publisherAuthority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [publisherAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('publisher'), publisherAuthority.publicKey.toBuffer()],
      program.programId
    );

    const [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault_authority')],
      program.programId
    );

    const [tokenVault] = PublicKey.findProgramAddressSync(
      [Buffer.from('token_vault')],
      program.programId
    );

    const publisherTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      publisherAuthority.publicKey
    );

    const vaultTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      vaultAuthority,
      true
    );

    const tx = await program.methods
      .withdrawUnbonded()
      .accounts({
        globalState,
        publisherAccount,
        vaultAuthority,
        tokenVault,
        publisherTokenAccount,
        vaultTokenAccount,
        publisherAuthority: publisherAuthority.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([publisherAuthority])
      .rpc();

    res.json({
      success: true,
      signature: tx
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

// ============================================================================
// GOVERNANCE ENDPOINTS
// ============================================================================

/**
 * POST /api/governance/proposals/create
 * Create a governance proposal
 */
app.post('/api/governance/proposals/create', async (req, res) => {
  try {
    const {
      proposerSecretKey,
      proposalType,
      description,
      tokenMintAddress
    } = req.body;

    const proposer = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(proposerSecretKey, 'base64'))
    );
    const tokenMint = new PublicKey(tokenMintAddress);

    const provider = getProvider({ publicKey: proposer.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [governanceState] = PublicKey.findProgramAddressSync(
      [Buffer.from('governance')],
      program.programId
    );

    const governanceAccount = await program.account.governanceState.fetch(governanceState);
    const proposalCount = governanceAccount.proposalCount;

    const [proposal] = PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), proposalCount.toArrayLike(Buffer, 'le', 8)],
      program.programId
    );

    const proposerTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      proposer.publicKey
    );

    // Convert proposal type to program format
    const proposalTypeEnum = convertProposalType(proposalType);

    const tx = await program.methods
      .createProposal(proposalTypeEnum, description)
      .accounts({
        globalState,
        governanceState,
        proposal,
        proposerTokenAccount,
        proposer: proposer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([proposer])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      proposalId: proposalCount.toString(),
      proposalAccount: proposal.toBase58()
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/governance/proposals/:proposalId/vote
 * Vote on a proposal
 */
app.post('/api/governance/proposals/:proposalId/vote', async (req, res) => {
  try {
    const { proposalId } = req.params;
    const { voterSecretKey, vote, tokenMintAddress } = req.body;

    const voter = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(voterSecretKey, 'base64'))
    );
    const tokenMint = new PublicKey(tokenMintAddress);

    const provider = getProvider({ publicKey: voter.publicKey });
    const program = getProgram(provider);

    const [proposal] = PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), new BN(proposalId).toArrayLike(Buffer, 'le', 8)],
      program.programId
    );

    const voterTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      voter.publicKey
    );

    const voteTypeEnum = { [vote.toLowerCase()]: {} };

    const tx = await program.methods
      .voteProposal(voteTypeEnum)
      .accounts({
        proposal,
        voterTokenAccount,
        voter: voter.publicKey,
      })
      .signers([voter])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      proposalId,
      vote
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/governance/proposals/:proposalId/execute
 * Execute a passed proposal
 */
app.post('/api/governance/proposals/:proposalId/execute', async (req, res) => {
  try {
    const { proposalId } = req.params;

    const provider = getProvider({ publicKey: web3.Keypair.generate().publicKey });
    const program = getProgram(provider);

    const [proposal] = PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), new BN(proposalId).toArrayLike(Buffer, 'le', 8)],
      program.programId
    );

    const [governanceState] = PublicKey.findProgramAddressSync(
      [Buffer.from('governance')],
      program.programId
    );

    const tx = await program.methods
      .executeProposal()
      .accounts({
        proposal,
        governanceState,
      })
      .rpc();

    res.json({
      success: true,
      signature: tx,
      proposalId
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/governance/proposals/:proposalId/execute-action
 * Execute the governance action after timelock
 */
app.post('/api/governance/proposals/:proposalId/execute-action', async (req, res) => {
  try {
    const { proposalId } = req.params;
    const { authoritySecretKey } = req.body;

    const authority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(authoritySecretKey, 'base64'))
    );

    const provider = getProvider({ publicKey: authority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const [proposal] = PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), new BN(proposalId).toArrayLike(Buffer, 'le', 8)],
      program.programId
    );

    const [governanceState] = PublicKey.findProgramAddressSync(
      [Buffer.from('governance')],
      program.programId
    );

    const [tokenVault] = PublicKey.findProgramAddressSync(
      [Buffer.from('token_vault')],
      program.programId
    );

    const tx = await program.methods
      .executeGovernanceAction()
      .accounts({
        globalState,
        proposal,
        governanceState,
        tokenVault,
        priceAccount: null,
        publisherAccount: null,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      proposalId
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

// ============================================================================
// EMERGENCY ENDPOINTS
// ============================================================================

/**
 * POST /api/emergency/pause
 * Emergency pause the system
 */
app.post('/api/emergency/pause', async (req, res) => {
  try {
    const { authoritySecretKey } = req.body;

    const authority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(authoritySecretKey, 'base64'))
    );

    const provider = getProvider({ publicKey: authority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const tx = await program.methods
      .emergencyPause()
      .accounts({
        globalState,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      message: 'System paused'
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/emergency/unpause
 * Emergency unpause the system
 */
app.post('/api/emergency/unpause', async (req, res) => {
  try {
    const { authoritySecretKey } = req.body;

    const authority = Keypair.fromSecretKey(
      Uint8Array.from(Buffer.from(authoritySecretKey, 'base64'))
    );

    const provider = getProvider({ publicKey: authority.publicKey });
    const program = getProgram(provider);

    const [globalState] = PublicKey.findProgramAddressSync(
      [Buffer.from('global_state')],
      program.programId
    );

    const tx = await program.methods
      .emergencyUnpause()
      .accounts({
        globalState,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    res.json({
      success: true,
      signature: tx,
      message: 'System unpaused'
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

// ============================================================================
// QUERY ENDPOINTS
// ============================================================================

/**
 * GET /api/prices/:symbol
 * Get current price for a symbol
 */
app.get('/api/prices/:symbol', async (req, res) => {
  try {
    const { symbol } = req.params;

    const provider = getProvider({ publicKey: web3.Keypair.generate().publicKey });
    const program = getProgram(provider);

    const [priceAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('price'), Buffer.from(symbol)],
      program.programId
    );

    const priceData = await program.account.priceAccount.fetch(priceAccount);

    res.json({
      success: true,
      symbol,
      price: priceData.aggregate.price.toString(),
      confidence: priceData.aggregate.confidence.toString(),
      exponent: priceData.exponent,
      timestamp: priceData.aggregate.timestamp.toString(),
      slot: priceData.aggregate.slot.toString(),
      status: Object.keys(priceData.aggregate.status)[0],
      publisherCount: priceData.publisherCount,
      ema: {
        price: priceData.ema.emaPrice.toString(),
        confidence: priceData.ema.emaConfidence.toString(),
        observations: priceData.ema.numObservations.toString()
      }
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * GET /api/publishers/:address
 * Get publisher information
 */
app.get('/api/publishers/:address', async (req, res) => {
  try {
    const { address } = req.params;
    const publisherAuthority = new PublicKey(address);

    const provider = getProvider({ publicKey: web3.Keypair.generate().publicKey });
    const program = getProgram(provider);

    const [publisherAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('publisher'), publisherAuthority.toBuffer()],
      program.programId
    );

    const publisherData = await program.account.publisherAccount.fetch(publisherAccount);

    res.json({
      success: true,
      publisher: {
        authority: publisherData.authority.toBase58(),
        name: publisherData.name,
        stakedAmount: publisherData.stakedAmount.toString(),
        reputation: publisherData.reputation.toString(),
        registeredAt: publisherData.registeredAt.toString(),
        slashCount: publisherData.slashCount,
        unbondingAmount: publisherData.unbondingAmount.toString(),
        unbondingStart: publisherData.unbondingStart.toString()
      }
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * GET /api/governance/proposals/:proposalId
 * Get proposal details
 */
app.get('/api/governance/proposals/:proposalId', async (req, res) => {
  try {
    const { proposalId } = req.params;

    const provider = getProvider({ publicKey: web3.Keypair.generate().publicKey });
    const program = getProgram(provider);

    const [proposal] = PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), new BN(proposalId).toArrayLike(Buffer, 'le', 8)],
      program.programId
    );

    const proposalData = await program.account.proposal.fetch(proposal);

    res.json({
      success: true,
      proposal: {
        proposalId: proposalData.proposalId.toString(),
        proposer: proposalData.proposer.toBase58(),
        description: proposalData.description,
        yesVotes: proposalData.yesVotes.toString(),
        noVotes: proposalData.noVotes.toString(),
        abstainVotes: proposalData.abstainVotes.toString(),
        startSlot: proposalData.startSlot.toString(),
        endSlot: proposalData.endSlot.toString(),
        executed: proposalData.executed,
        executionTime: proposalData.executionTime.toString(),
        proposalType: proposalData.proposalType
      }
    });
  } catch (error) {
    res.status(500).json({ success: false, error: error.message });
  }
});

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function convertProposalType(proposalType) {
  const { type, ...params } = proposalType;
  
  switch (type) {
    case 'UpdateRewardRate':
      return { updateRewardRate: { newRate: new BN(params.newRate) } };
    case 'UpdateMinPublishers':
      return { 
        updateMinPublishers: { 
          feed: new PublicKey(params.feed), 
          newMin: params.newMin 
        } 
      };
    case 'SlashPublisher':
      return { 
        slashPublisher: { 
          publisher: new PublicKey(params.publisher), 
          percentage: params.percentage 
        } 
      };
    case 'EmergencyPause':
      return { emergencyPause: {} };
    case 'EmergencyUnpause':
      return { emergencyUnpause: {} };
    case 'UpdateGovernanceParams':
      return { 
        updateGovernanceParams: { 
          proposalThreshold: params.proposalThreshold ? new BN(params.proposalThreshold) : null,
          votingPeriod: params.votingPeriod ? new BN(params.votingPeriod) : null,
          quorumPercentage: params.quorumPercentage || null,
          timelockDuration: params.timelockDuration ? new BN(params.timelockDuration) : null,
        } 
      };
    default:
      throw new Error('Invalid proposal type');
  }
}

// ============================================================================
// SERVER START
// ============================================================================

app.listen(PORT, () => {
  console.log(`Oracle API Server running on port ${PORT}`);
  console.log(`RPC URL: ${RPC_URL}`);
  console.log(`Program ID: ${PROGRAM_ID.toBase58()}`);
});