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
    TOKEN_PROGRAM_ID,
    createAssociatedTokenAccountIdempotentInstruction,
    createInitializeMint2Instruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync,
    getMinimumBalanceForRentExemptMint,
    getAccount,
    createAssociatedTokenAccount,
    createAssociatedTokenAccountInstruction,
    ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { randomBytes } from "crypto";
import assert from "assert";

describe("dao-voting-program", () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const provider = anchor.getProvider() as anchor.AnchorProvider;
    const connection = provider.connection;
    const program = anchor.workspace.DaoVotingProgram as Program<DaoVotingProgram>;
    const tokenProgram = TOKEN_PROGRAM_ID;

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

    // let mintKeypair: Keypair;
    // let mint: PublicKey;

    const seed = new BN(randomBytes(8));
    const [admin, user1, user2] = Array.from({ length: 3 }, () => Keypair.generate());

   
    let adminAta: PublicKey;
    let user1Ata: PublicKey;
    let user2Ata: PublicKey;
    let configPda: PublicKey;
    let treasuryPda: PublicKey;
    let authPda: PublicKey;

    let user1StakeAta: PublicKey;
    let user1StakeState: PublicKey;
    let user1MemberState: PublicKey;

    let proposalPda: PublicKey;
    let votePda: PublicKey;
    let mintPda: PublicKey;
    const proposalId = new BN(1);

    before(async () => {
        console.log("Starting before hook");
        // Airdrop SOL to admin and users
        await Promise.all(
            [admin, user1, user2].map(async (kp) => {
                await connection.requestAirdrop(kp.publicKey, 100 * LAMPORTS_PER_SOL)
                    .then(confirm)
                    .then(log);
            })
        );

        console.log("Deriving PDAs");
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

        [mintPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("mint"), configPda.toBuffer()],
            program.programId
        );


        console.log("Creating mint");
        // Create mint
        const mintKeypair = Keypair.generate();
        mintPda = mintKeypair.publicKey;

        const lamports = await getMinimumBalanceForRentExemptMint(connection);
        const createAccountIx = SystemProgram.createAccount({
            fromPubkey: provider.wallet.publicKey,
            newAccountPubkey: mintPda,
            space: MINT_SIZE,
            lamports,
            programId: TOKEN_PROGRAM_ID,
        });

        const initializeMintIx = createInitializeMint2Instruction(
            mintPda,
            6,
            authPda,
            null,
            TOKEN_PROGRAM_ID
        );
        const tx = new Transaction().add(createAccountIx, initializeMintIx);

        try {
            await provider.sendAndConfirm(tx, [mintKeypair]).then(log);
            console.log("Mint created successfully");
        } catch (error) {
            console.error("Error creating mint:", error);
            throw error;
        }

        // const tx = new Transaction();
        // tx.add(
        //     SystemProgram.createAccount({
        //         fromPubkey: admin.publicKey,
        //         newAccountPubkey: mintPda,
        //         space: MINT_SIZE,
        //         lamports: await getMinimumBalanceForRentExemptMint(connection),
        //         programId: TOKEN_PROGRAM_ID,
        //     }),
        //     createInitializeMint2Instruction(
        //         mintPda,
        //         6,
        //         authPda,
        //         null,
        //         TOKEN_PROGRAM_ID
        //     )
        // );

        // // Send and confirm transaction
        // const txSignature = await connection.sendTransaction(tx, [admin]);
        // await connection.confirmTransaction(txSignature, 'confirmed');
        // await log(txSignature);

        // Create ATAs
        // adminAta = await createAssociatedTokenAccount(
        //     connection,
        //     admin,
        //     mintPda,
        //     admin.publicKey
        // );

        // user1Ata = await createAssociatedTokenAccount(
        //     connection,
        //     admin,
        //     mintPda,
        //     user1.publicKey
        //   );

        // user2Ata = await createAssociatedTokenAccount(
        //     connection,
        //     admin,
        //     mintPda,
        //     user2.publicKey
        //   );

        console.log("Creating ATAs")
        // Create ATAs
        adminAta = getAssociatedTokenAddressSync(mintPda, admin.publicKey);
        user1Ata = getAssociatedTokenAddressSync(mintPda, user1.publicKey);
        user2Ata = getAssociatedTokenAddressSync(mintPda, user2.publicKey);

        const createAtasTx = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                adminAta,
                admin.publicKey,
                mintPda
            ),
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                user1Ata,
                user1.publicKey,
                mintPda
            ),
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                user2Ata,
                user2.publicKey,
                mintPda
            )
        );

        // const createAtasTxSignature = await connection.sendTransaction(createAtasTx, [admin]);
        // await connection.confirmTransaction(createAtasTxSignature, 'confirmed');
        // await log(createAtasTxSignature);

        try {
            await provider.sendAndConfirm(createAtasTx, []).then(log);
            console.log("ATAs created successfully");
        } catch (error) {
            console.error("Error creating ATAs:", error);
            throw error;
        }

        console.log("Deriving other PDAs");

        //Derive other PDAs
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

        [proposalPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("proposal"), configPda.toBuffer(), proposalId.toArrayLike(Buffer, "le", 8)],
            program.programId
        );

        [votePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("vote"), proposalPda.toBuffer(), user1.publicKey.toBuffer()],
            program.programId
        );

        console.log("Before hook completed successfully");
        // // Create ATAs
        // adminAta = getAssociatedTokenAddressSync(mint, admin.publicKey);
        // user1Ata = getAssociatedTokenAddressSync(mint, user1.publicKey);
        // user2Ata = getAssociatedTokenAddressSync(mint, user2.publicKey);

        // await connection.confirmTransaction(txSignature, 'confirmed');
        // await log(txSignature);

        // Assert admin received tokens
        // const adminAccount = await getAccount(connection, adminAta);
        // assert.equal(adminAccount.amount.toString(), "1000000000", "Admin should have 1e9 tokens");
    });

    it("Initialize DAO", async () => {
        [mintPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("mint"), configPda.toBuffer()],
            program.programId
        );
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
                initializer: provider.wallet.publicKey,
                mint: mintPda,
                config: configPda,
                treasury: treasuryPda,
                auth: authPda,
                tokenProgram,
                systemProgram: SystemProgram.programId,
            })//.rpc.then(confirm).then(log)
        .signers([admin])
        .rpc()
        .then(confirm)
        .then(log);

        const config = await program.account.daoSetup.fetch(configPda);
        assert.ok(config, "config should be initialized")
        // assert.equal(config.seed.toString(), seed.toString(), "Config seed should match");
        // assert.equal(config.issuePrice.toString(), (1 * LAMPORTS_PER_SOL).toString(), "Issue price should match");
        // assert.equal(config.issueAmount.toString(), "100", "Issue amount should match");
        // assert.equal(config.proposalFee.toString(), (0.1 * LAMPORTS_PER_SOL).toString(), "Proposal fee should match");
    });

    it("Issue tokens", async () => {
        // Fetch config to ensure it's initialized
        const config = await program.account.daoSetup.fetch(configPda);
        assert.ok(config, "Config should be initialized");
        const initialBalance = await connection.getBalance(user1.publicKey);

        await program.methods
            .issueTokens()
            .accounts({
                initializer: user1.publicKey,
                initializerAta: user1Ata,
                auth: authPda,
                treasury: treasuryPda,
                mint: mintPda,
                config: configPda,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        const user1Account = await getAccount(connection, user1Ata);
        //assert.equal(user1Account.amount.toString(), "100", "User1 should have 100 tokens");

        const finalBalance = await connection.getBalance(user1.publicKey);
        //assert(finalBalance < initialBalance, "User1's SOL balance should have decreased");

        const treasuryBalance = await connection.getBalance(treasuryPda);
        assert(treasuryBalance > 0, "Treasury should have received SOL");
    });

    it("Initialize stake", async () => {
        await program.methods
            .initStake()
            .accounts({
                owner: user1.publicKey,
                ownerAta: user1Ata,
                stakeAta: user1StakeAta,
                stakeAuth: authPda,
                mint: mintPda,
                stakeState: user1StakeState,
                config: configPda,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            // .then(confirm)
            // .then(log);

        const stakeState = await program.account.stakeState.fetch(user1StakeState);
        //assert.equal(stakeState.owner.toBase58(), user1.publicKey.toBase58(), "Stake state owner should be user1");
        assert.ok(stakeState, "Stake should be initialized");
    });

    it("Stake tokens", async () => {
        const amount = new BN(50);
        await program.methods
            .stakeTokens(amount)
            .accounts({
                owner: user1.publicKey,
                ownerAta: user1Ata,
                stakeAta: user1StakeAta,
                mint: mintPda,
                auth: authPda,
                stakeState: user1StakeState,
                memberState: user1MemberState,
                config: configPda,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        const stakeAccount = await getAccount(connection, user1StakeAta);
        assert.equal(stakeAccount.amount.toString(), "50", "Stake account should have 50 tokens");

        const userAccount = await getAccount(connection, user1Ata);
        assert.equal(userAccount.amount.toString(), "50", "User account should have 50 tokens left");
    });

    it("Create proposal", async () => {
        await program.methods
            .createProposal(
                proposalId,
                "Test Proposal",
                "https://example.com/proposal",
                { vote: {} },
                new BN(10), // threshold
                new BN(100) // expiry
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

        const proposal = await program.account.proposal.fetch(proposalPda);
        assert.equal(proposal.id.toString(), "1", "Proposal ID should be 1");
        assert.equal(proposal.name, "Test Proposal", "Proposal name should match");
    });

    it("Vote on proposal", async () => {
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

        const vote = await program.account.voteState.fetch(votePda);
        assert.equal(vote.amount.toString(), "50", "Vote amount should be 50");
        assert.deepEqual(vote.voteType, { yes: {} }, "Vote type should be 'yes'");
    });

    it("Get proposal results", async () => {
        const results = await program.methods
            .getProposalResults()
            .accounts({
                user: user1.publicKey,
                proposal: proposalPda,
                config: configPda,
            })
            .view();

        assert.equal(results.yesVotes.toString(), "50", "Yes votes should be 50");
        assert.equal(results.noVotes.toString(), "0", "No votes should be 0");
        assert.equal(results.abstainVotes.toString(), "0", "Abstain votes should be 0");
    });

    it("Remove vote", async () => {
        // Add a delay to avoid InvalidSlot error
        await new Promise(resolve => setTimeout(resolve, 1000));

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

        const proposal = await program.account.proposal.fetch(proposalPda);
        assert.equal(proposal.votes.toString(), "0", "Total votes should be 0 after removal");
    });

    it("Executes a proposal", async () => {
        // Vote again to meet the quorum
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

        // Generate a random payee for this test
        const payee = Keypair.generate().publicKey;

        await program.methods
            .executeProposal()
            .accounts({
                initializer: user1.publicKey,
                payee: payee,
                proposal: proposalPda,
                treasury: treasuryPda,
                config: configPda,
                proposerState: user1MemberState,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        const proposalAfter = await program.account.proposal.fetch(proposalPda);
        assert.deepEqual(proposalAfter.result, { succeeded: {} }, "Proposal result should be 'succeeded'");

        // Fetch and check the proposer's member state
        const proposerStateAfter = await program.account.memberState.fetch(user1MemberState);
        assert(proposerStateAfter.rewardPoints.gt(new BN(0)), "Proposer should have received reward points");
        assert(proposerStateAfter.successfulProposals.eq(new BN(1)), "Proposer should have 1 successful proposal");
    });

    it("Unstake tokens", async () => {
        const amount = new BN(25);
        await program.methods
            .unstakeTokens(amount)
            .accounts({
                owner: user1.publicKey,
                ownerAta: user1Ata,
                stakeAta: user1StakeAta,
                auth: authPda,
                mint: mintPda,
                stakeState: user1StakeState,
                config: configPda,
                memberState: user1MemberState,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        const stakeAccount = await getAccount(connection, user1StakeAta);
        assert.equal(stakeAccount.amount.toString(), "25", "Stake account should have 25 tokens left");

        const userAccount = await getAccount(connection, user1Ata);
        assert.equal(userAccount.amount.toString(), "75", "User account should have 75 tokens after unstaking");
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

        assert.equal(memberState.address.toBase58(), user1.publicKey.toBase58(), "Member address should match");
        assert(memberState.rewardPoints.gt(new BN(0)), "Member should have some reward points");
        assert.equal(memberState.totalVotesCast.toString(), "2", "Member should have cast 2 votes");
        assert.equal(memberState.proposalsCreated.toString(), "1", "Member should have created 1 proposal");
    });

    it("Close stake account", async () => {
        // Unstake remaining tokens
        const stakeAccount = await getAccount(connection, user1StakeAta);
        const remainingStake = new BN(stakeAccount.amount.toString());

        await program.methods
            .unstakeTokens(remainingStake)
            .accounts({
                owner: user1.publicKey,
                ownerAta: user1Ata,
                stakeAta: user1StakeAta,
                auth: authPda,
                mint: mintPda,
                stakeState: user1StakeState,
                config: configPda,
                memberState: user1MemberState,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        const initialBalance = await connection.getBalance(user1.publicKey);

        await program.methods
            .closeStakeAccount()
            .accounts({
                owner: user1.publicKey,
                stakeAta: user1StakeAta,
                stakeAuth: authPda,
                mint: mintPda,
                stakeState: user1StakeState,
                config: configPda,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        const finalBalance = await connection.getBalance(user1.publicKey);
        assert(finalBalance > initialBalance, "User's SOL balance should have increased after closing stake account");

        await assert.rejects(
            getAccount(connection, user1StakeAta),
            /Account does not exist/,
            "Stake ATA should be closed"
        );
    });

    it("Cleanup proposal", async () => {
        // Remove vote before cleanup
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

        await assert.rejects(
            program.account.proposal.fetch(proposalPda),
            /Account does not exist/,
            "Proposal account should be closed"
        );
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

        await assert.rejects(
            program.account.voteState.fetch(votePda),
            /Account does not exist/,
            "Vote account should be closed"
        );
    });

    it("Fails to vote twice", async () => {
        // First, create a new proposal
        const proposalId2 = new BN(2);
        const [proposalPda2] = PublicKey.findProgramAddressSync(
            [Buffer.from("proposal"), configPda.toBuffer(), proposalId2.toArrayLike(Buffer, "le", 8)],
            program.programId
        );

        await program.methods
            .createProposal(
                proposalId2,
                "Test Proposal 2",
                "https://example.com/proposal2",
                { vote: {} },
                new BN(10),
                new BN(100)
            )
            .accounts({
                owner: user1.publicKey,
                stakeState: user1StakeState,
                proposal: proposalPda2,
                memberState: user1MemberState,
                treasury: treasuryPda,
                config: configPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        // Vote once
        const [votePda2] = PublicKey.findProgramAddressSync(
            [Buffer.from("vote"), proposalPda2.toBuffer(), user1.publicKey.toBuffer()],
            program.programId
        );

        await program.methods
            .vote(new BN(25), { yes: {} })
            .accounts({
                owner: user1.publicKey,
                stakeState: user1StakeState,
                proposal: proposalPda2,
                vote: votePda2,
                memberState: user1MemberState,
                config: configPda,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        // Try to vote again
        await assert.rejects(
            program.methods
                .vote(new BN(25), { yes: {} })
                .accounts({
                    owner: user1.publicKey,
                    stakeState: user1StakeState,
                    proposal: proposalPda2,
                    vote: votePda2,
                    memberState: user1MemberState,
                    config: configPda,
                    systemProgram: SystemProgram.programId,
                })
                .signers([user1])
                .rpc(),
            /AlreadyVoted/,
            "Should not be able to vote twice on the same proposal"
        );
    });

    it("Fails to create proposal without sufficient stake", async () => {
        // Unstake all tokens first
        const stakeAccount = await getAccount(connection, user1StakeAta);
        const remainingStake = new BN(stakeAccount.amount.toString());

        await program.methods
            .unstakeTokens(remainingStake)
            .accounts({
                owner: user1.publicKey,
                ownerAta: user1Ata,
                stakeAta: user1StakeAta,
                auth: authPda,
                mint: mintPda,
                stakeState: user1StakeState,
                config: configPda,
                memberState: user1MemberState,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        // Try to create a proposal
        const proposalId3 = new BN(3);
        const [proposalPda3] = PublicKey.findProgramAddressSync(
            [Buffer.from("proposal"), configPda.toBuffer(), proposalId3.toArrayLike(Buffer, "le", 8)],
            program.programId
        );

        await assert.rejects(
            program.methods
                .createProposal(
                    proposalId3,
                    "Test Proposal 3",
                    "https://example.com/proposal3",
                    { vote: {} },
                    new BN(10),
                    new BN(100)
                )
                .accounts({
                    owner: user1.publicKey,
                    stakeState: user1StakeState,
                    proposal: proposalPda3,
                    memberState: user1MemberState,
                    treasury: treasuryPda,
                    config: configPda,
                    systemProgram: SystemProgram.programId,
                })
                .signers([user1])
                .rpc(),
            /InsufficientStake/,
            "Should not be able to create a proposal without sufficient stake"
        );
    });
});