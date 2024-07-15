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
    createAssociatedTokenAccountInstruction,
    createInitializeMint2Instruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync,
    getMinimumBalanceForRentExemptMint,
    getAccount,
    ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { randomBytes } from "crypto";
import assert from "assert";

describe("dao-voting-program", () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const provider = anchor.getProvider() as anchor.AnchorProvider;
    const connection = provider.connection;
    const program = anchor.workspace.DaoVotingProgram as Program<DaoVotingProgram>;

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
    const mintKeypair = Keypair.generate();
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
    const proposalId = new BN(1);

    it("Initialize environment", async () => {
        // Airdrop SOL to admin and users
        await Promise.all(
            [admin, user1, user2].map(async (kp) => {
                await connection.requestAirdrop(kp.publicKey, 100 * LAMPORTS_PER_SOL)
                    .then(confirm)
                    .then(log);
            })
        );

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

        // Create mint
        const lamports = await getMinimumBalanceForRentExemptMint(connection);
        const createAccountIx = SystemProgram.createAccount({
            fromPubkey: provider.wallet.publicKey,
            newAccountPubkey: mintKeypair.publicKey,
            space: MINT_SIZE,
            lamports,
            programId: TOKEN_PROGRAM_ID,
        });

        const initializeMintIx = createInitializeMint2Instruction(
            mintKeypair.publicKey,
            6,
            authPda,
            null,
            TOKEN_PROGRAM_ID
        );
        const tx = new Transaction().add(createAccountIx, initializeMintIx);

        await provider.sendAndConfirm(tx, [mintKeypair]).then(log);

        // Create ATAs
        adminAta = getAssociatedTokenAddressSync(mintKeypair.publicKey, admin.publicKey);
        user1Ata = getAssociatedTokenAddressSync(mintKeypair.publicKey, user1.publicKey);
        user2Ata = getAssociatedTokenAddressSync(mintKeypair.publicKey, user2.publicKey);

        const createAtasTx = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                adminAta,
                admin.publicKey,
                mintKeypair.publicKey
            ),
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                user1Ata,
                user1.publicKey,
                mintKeypair.publicKey
            ),
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                user2Ata,
                user2.publicKey,
                mintKeypair.publicKey
            )
        );

        await provider.sendAndConfirm(createAtasTx, []).then(log);

        // Derive other PDAs
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
    });

    it("Initialize DAO", async () => {
        await program.methods
            .initialize(
                seed,
                new BN(1 * LAMPORTS_PER_SOL),
                new BN(100),
                new BN(0.1 * LAMPORTS_PER_SOL),
                new BN(1e6),
                new BN(100),
                new BN(100000)
            )
            .accounts({
                initializer: provider.wallet.publicKey,
                mint: mintKeypair.publicKey,
                config: configPda,
                treasury: treasuryPda,
                auth: authPda,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .rpc()
            .then(confirm)
            .then(log);

        const config = await program.account.daoSetup.fetch(configPda);
        assert.ok(config, "Config should be initialized");
    });

    it("Issue tokens", async () => {
        const initialBalance = await connection.getBalance(user1.publicKey);

        await program.methods
            .issueTokens()
            .accounts({
                initializer: user1.publicKey,
                initializerAta: user1Ata,
                auth: authPda,
                treasury: treasuryPda,
                mint: mintKeypair.publicKey,
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
        assert.equal(user1Account.amount.toString(), "100", "User1 should have 100 tokens");

        const finalBalance = await connection.getBalance(user1.publicKey);
        assert(finalBalance < initialBalance, "User1's SOL balance should have decreased");

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
                mint: mintKeypair.publicKey,
                stakeState: user1StakeState,
                config: configPda,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc()
            .then(confirm)
            .then(log);

        const stakeState = await program.account.stakeState.fetch(user1StakeState);
        assert.ok(stakeState, "Stake state should be initialized");
    });

    it("Stake tokens", async () => {
        const amount = new BN(50);
        await program.methods
            .stakeTokens(amount)
            .accounts({
                owner: user1.publicKey,
                ownerAta: user1Ata,
                stakeAta: user1StakeAta,
                mint: mintKeypair.publicKey,
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
                new BN(10),
                new BN(100)
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
                mint: mintKeypair.publicKey,
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
                mint: mintKeypair.publicKey,
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
                mint: mintKeypair.publicKey,
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
                mint: mintKeypair.publicKey,
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