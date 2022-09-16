import {createAssociatedTokenAccount, createMint, mintTo} from '@solana/spl-token';
import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {Challenge} from "../target/types/challenge";
import {
    cancelWithdrawal,
    createPool,
    deposit,
    initIfNeeded,
    processWithdrawalQueue,
    requestWithdrawal
} from "./utils/utils";

describe("Challenge demo", () => {
    // -- Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Challenge as Program<Challenge>;
    let provider = anchor.getProvider();
    let connection = provider.connection;

    it("Demonstrates how to interact with the challenge", async () => {
        const adminKeypair = anchor.web3.Keypair.generate();
        const adminAirdropTx = await connection.requestAirdrop(adminKeypair.publicKey, 1000000000);
        await connection.confirmTransaction(adminAirdropTx, "confirmed");

        // -- Create token mint
        console.log("Creating legit mint");
        let mintPubkey = await createMint(
            connection,
            adminKeypair,
            adminKeypair.publicKey,
            null,
            18,
            undefined,
            {commitment: "confirmed"},
        );

        // -- Init the program
        console.log("Initializing program");
        let configPubkey = await initIfNeeded(program, adminKeypair);

        // -- Create pool
        console.log("Creating legitimate pool");
        let pool = await createPool(program, configPubkey, adminKeypair, mintPubkey);

        // -- Create admin token accounts
        let adminTokenAccountPubkey = await createAssociatedTokenAccount(
            connection,
            adminKeypair,
            mintPubkey,
            adminKeypair.publicKey,
        );

        let adminRedeemTokenAccountPubkey = await createAssociatedTokenAccount(
            connection,
            adminKeypair,
            pool.redeemMintPubkey,
            adminKeypair.publicKey,
        );

        // -- Mint tokens
        console.log("Minting tokens")
        await mintTo(
            connection,
            adminKeypair,
            mintPubkey,
            adminTokenAccountPubkey,
            adminKeypair,
            1000,
            undefined,
            {commitment: "confirmed"},
        );

        // -- Deposit tokens to pool
        console.log("Depositing funds to pool")
        await deposit(program, pool, adminKeypair, adminTokenAccountPubkey, adminRedeemTokenAccountPubkey);

        // Request withdrawals
        console.log("Requesting withdrawals")
        let adminWithdrawal1 = await requestWithdrawal(program, pool, 10, adminKeypair, adminRedeemTokenAccountPubkey);
        let adminWithdrawal2 = await requestWithdrawal(program, pool, 10, adminKeypair, adminRedeemTokenAccountPubkey);

        // Cancel one withdrawal
        console.log("Cancelling one of the withdrawals");
        await cancelWithdrawal(program, pool, adminKeypair, adminWithdrawal1.nodePubkey);

        // Process the withdrawal queue
        console.log("Processing withdrawal queue")
        await processWithdrawalQueue(program, pool);
    });
});
