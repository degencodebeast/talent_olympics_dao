// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { DaoVotingProgram } from "../target/types/dao_voting_program";

// describe("dao-voting-program", () => {
//   // Configure the client to use the local cluster.
//   anchor.setProvider(anchor.AnchorProvider.env());

//   const program = anchor.workspace.DaoVotingProgram as Program<DaoVotingProgram>;

//   it("Is initialized!", async () => {
//     // Add your test here.
//     const tx = await program.methods.initialize().rpc();
//     console.log("Your transaction signature", tx);
//   });
// });

import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { DaoVotingProgram } from "../target/types/dao_voting_program";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from "@solana/spl-token";
import { randomBytes } from "crypto";

describe("dao-voting-program", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.DaoVotingProgram as Program<DaoVotingProgram>;
  const tokenProgram = TOKEN_2022_PROGRAM_ID;

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  const seed = new BN(randomBytes(8));
  const [admin, user1, user2] = Array.from({ length: 3 }, () => Keypair.generate());

  let mintKeypair: Keypair;
  let mint: PublicKey;
  let adminAta: PublicKey;
  let user1Ata: PublicKey;
  let user2Ata: PublicKey;
  let configPda: PublicKey;
  let treasuryPda: PublicKey;
  let authPda: PublicKey;

  before(async () => {
    // Airdrop SOL to admin and users
    await Promise.all(
      [admin, user1, user2].map(async (kp) => {
        await connection.requestAirdrop(kp.publicKey, 100 * LAMPORTS_PER_SOL)
          .then(confirm)
          .then(log);
      })
    );

    // Create mint
    mintKeypair = Keypair.generate();
    mint = mintKeypair.publicKey;

    // Derive PDAs
    [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [treasuryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury"), configPda.toBuffer()],
      program.programId
    );
    [authPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("auth"), configPda.toBuffer()],
      program.programId
    );

    // Create ATAs
    adminAta = getAssociatedTokenAddressSync(mint, admin.publicKey);
    user1Ata = getAssociatedTokenAddressSync(mint, user1.publicKey);
    user2Ata = getAssociatedTokenAddressSync(mint, user2.publicKey);

    // Create mint and mint tokens to admin
    const lamports = await getMinimumBalanceForRentExemptMint(connection);
    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: mint,
        lamports,
        space: MINT_SIZE,
        programId: tokenProgram,
      }),
      createInitializeMint2Instruction(mint, 6, authPda, null, tokenProgram),
      createAssociatedTokenAccountIdempotentInstruction(
        admin.publicKey,
        adminAta,
        admin.publicKey,
        mint
      ),
      createMintToInstruction(mint, adminAta, authPda, 1e9, [], tokenProgram)
    );

    await provider.sendAndConfirm(tx, [admin, mintKeypair]).then(log);
  });

  it("Initialize DAO", async () => {
    await program.methods
      .initialize(
        seed,
        new BN(1 * LAMPORTS_PER_SOL), // issue_price
        new BN(100), // issue_amount
        new BN(0.1 * LAMPORTS_PER_SOL), // proposal_fee
        new BN(1e6), // max_supply
        new BN(100), // min_quorum
        new BN(100000) // max_expiry
      )
      .accounts({
        initializer: admin.publicKey,
        mint,
        config: configPda,
        treasury: treasuryPda,
        auth: authPda,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Issue tokens", async () => {
    await program.methods
      .issueTokens()
      .accounts({
        initializer: user1.publicKey,
        initializerAta: user1Ata,
        auth: authPda,
        treasury: treasuryPda,
        mint,
        config: configPda,
        tokenProgram,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  let user1StakeAta: PublicKey;
  let user1StakeState: PublicKey;
  let user1MemberState: PublicKey;

  it("Initialize stake", async () => {
    [user1StakeAta] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), configPda.toBuffer(), user1.publicKey.toBuffer()],
      program.programId
    );
    [user1StakeState] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), configPda.toBuffer(), user1.publicKey.toBuffer()],
      program.programId
    );
    [user1MemberState] = PublicKey.findProgramAddressSync(
      [Buffer.from("member"), configPda.toBuffer(), user1.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initStake()
      .accounts({
        owner: user1.publicKey,
        ownerAta: user1Ata,
        stakeAta: user1StakeAta,
        stakeAuth: authPda,
        mint,
        stakeState: user1StakeState,
        memberState: user1MemberState,
        config: configPda,
        tokenProgram,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Stake tokens", async () => {
    const amount = new BN(50);
    await program.methods
      .stakeTokens(amount)
      .accounts({
        owner: user1.publicKey,
        ownerAta: user1Ata,
        stakeAta: user1StakeAta,
        stakeAuth: authPda,
        mint,
        stakeState: user1StakeState,
        memberState: user1MemberState,
        config: configPda,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  let proposalPda: PublicKey;
  const proposalId = new BN(1);

  it("Create proposal", async () => {
    [proposalPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("proposal"), configPda.toBuffer(), proposalId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    await program.methods
      .createProposal(
        proposalId,
        "Test Proposal",
        "https://example.com/proposal",
        { vote: {} },
        new BN(10), // threshold
        new BN(100) // amount
      )
      .accounts({
        owner: user1.publicKey,
        stakeState: user1StakeState,
        proposal: proposalPda,
        memberState: user1MemberState,
        treasury: treasuryPda,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  let votePda: PublicKey;

  it("Vote on proposal", async () => {
    [votePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vote"), proposalPda.toBuffer(), user1.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .vote(new BN(50), { yes: {} })
      .accounts({
        owner: user1.publicKey,
        stakeState: user1StakeState,
        proposal: proposalPda,
        vote: votePda,
        memberState: user1MemberState,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Get proposal results", async () => {
    const results = await program.methods
      .getProposalResults()
      .accounts({
        proposal: proposalPda,
        config: configPda,
      })
      .view();

    console.log("Proposal Results:", results);
  });

  it("Remove vote", async () => {
    await program.methods
      .removeVote()
      .accounts({
        owner: user1.publicKey,
        stakeState: user1StakeState,
        proposal: proposalPda,
        vote: votePda,
        memberState: user1MemberState,
        treasury: treasuryPda,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Execute proposal", async () => {
    await program.methods
      .executeProposal()
      .accounts({
        initializer: user1.publicKey,
        proposal: proposalPda,
        treasury: treasuryPda,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Unstake tokens", async () => {
    const amount = new BN(25);
    await program.methods
      .unstakeTokens(amount)
      .accounts({
        owner: user1.publicKey,
        ownerAta: user1Ata,
        stakeAta: user1StakeAta,
        stakeAuth: authPda,
        mint,
        stakeState: user1StakeState,
        config: configPda,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Get member state", async () => {
    const memberState = await program.methods
      .getMemberState()
      .accounts({
        member: user1.publicKey,
        memberState: user1MemberState,
        config: configPda,
      })
      .view();

    console.log("Member State:", memberState);
  });

  it("Close stake account", async () => {
    await program.methods
      .closeStakeAccount()
      .accounts({
        owner: user1.publicKey,
        stakeAta: user1StakeAta,
        stakeAuth: authPda,
        mint,
        stakeState: user1StakeState,
        config: configPda,
        tokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Cleanup proposal", async () => {
    await program.methods
      .cleanupProposal()
      .accounts({
        initializer: user1.publicKey,
        proposal: proposalPda,
        treasury: treasuryPda,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Cleanup vote", async () => {
    await program.methods
      .cleanupVote()
      .accounts({
        owner: user1.publicKey,
        stakeState: user1StakeState,
        proposal: proposalPda,
        vote: votePda,
        memberState: user1MemberState,
        treasury: treasuryPda,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc()
      .then(confirm)
      .then(log);
  });
});
