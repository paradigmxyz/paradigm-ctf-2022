import {Keypair, PublicKey} from "@solana/web3.js";
import {getAssociatedTokenAddress} from '@solana/spl-token';
import {BN} from "@project-serum/anchor";

type Pool = {
    pubkey: PublicKey,
    bump: number,
    redeemMintPubkey: PublicKey,
    redeemMintBump: number,
    tokenAccountPubkey: PublicKey,
    tokenAccountBump: number,
    withdrawalQueueHeaderPubkey: PublicKey,
    withdrawalQueueHeaderBump: number,
    tokenMintPubkey: PublicKey,
}

export async function initIfNeeded(program, adminKeypair) {
    const [configPubkey, ] = PublicKey.findProgramAddressSync(
        [Buffer.from("CONFIG_SEED")],
        program.programId
    );
    let maybeConfig = await program.account.config.fetchNullable(configPubkey);
    if (maybeConfig != null) {
        return configPubkey;
    }

    await program.methods.initialize()
        .accounts({
            config: configPubkey,
            signer: adminKeypair.publicKey,
        })
        .signers([adminKeypair])
        .rpc({commitment: "confirmed"});

    return configPubkey;
}

export async function createPool(program, configPubkey, adminKeypair, mintPubkey): Promise<Pool> {
    let configAccount = await program.account.config.fetch(configPubkey, "confirmed");
    let [poolPubkey, poolBump] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("POOL_SEED"),
            configAccount.nextFreePoolSeed.toArrayLike(Buffer, 'le', 8),
        ],
        program.programId
    );

    let [poolRedeemMintAccountPubkey, poolRedeemMintAccountBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("POOL_REDEEM_TOKENS_MINT_SEED"), poolPubkey.toBuffer()],
        program.programId
    );
    let [poolTokenAccountPubkey, poolTokenAccountBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("POOL_TOKEN_ACCOUNT_SEED"), poolPubkey.toBuffer()],
        program.programId
    );
    let [withdrawalQueueHeaderAccountPubkey, withdrawalQueueHeaderAccountBump]  = PublicKey.findProgramAddressSync(
        [Buffer.from("POOL_QUEUE_HEADER_SEED"), poolPubkey.toBuffer()],
        program.programId
    );

    await program.methods.createPool()
        .accounts({
            config: configPubkey,
            pool: poolPubkey,
            poolRedeemTokensMint: poolRedeemMintAccountPubkey,
            tokenMint: mintPubkey,
            poolTokenAccount: poolTokenAccountPubkey,
            withdrawalQueueHeader: withdrawalQueueHeaderAccountPubkey,
            signer: adminKeypair.publicKey,
        })
        .signers([adminKeypair])
        .rpc({commitment: "confirmed"});

    return {
        pubkey: poolPubkey,
        bump: poolBump,
        redeemMintPubkey: poolRedeemMintAccountPubkey,
        redeemMintBump: poolRedeemMintAccountBump,
        tokenAccountPubkey: poolTokenAccountPubkey,
        tokenAccountBump: poolTokenAccountBump,
        withdrawalQueueHeaderPubkey: withdrawalQueueHeaderAccountPubkey,
        withdrawalQueueHeaderBump: withdrawalQueueHeaderAccountBump,
        tokenMintPubkey: mintPubkey,
    }
}

export async function deposit(program, pool: Pool, signerKeypair, signerTokenAccountPubkey, signerRedeemTokenAccountPubkey) {
    await program.methods.deposit(new BN(100))
        .accounts({
            pool: pool.pubkey,
            poolRedeemTokensMint: pool.redeemMintPubkey,
            tokenMint: pool.tokenMintPubkey,
            poolTokenAccount: pool.tokenAccountPubkey,
            user: signerKeypair.publicKey,
            userTokenAccount: signerTokenAccountPubkey,
            userRedeemTokenAccount: signerRedeemTokenAccountPubkey,
        })
        .signers([signerKeypair])
        .rpc({commitment: "confirmed"});
}

export async function requestWithdrawal(program, pool: Pool, amount: number, signerKeypair, signerRedeemTokenAccountPubkey) {
    let withdrawalQueueHeader = await program.account.withdrawalQueueHeader.fetch(pool.withdrawalQueueHeaderPubkey);
    let [withdrawalQueueNewNodeAccountPubkey, withdrawalQueueNewNodeAccountBump] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("POOL_QUEUE_NODE_SEED"),
            pool.withdrawalQueueHeaderPubkey.toBuffer(),
            withdrawalQueueHeader.nonce.toArrayLike(Buffer, "le", 8)
        ],
        program.programId,
    );

    let remainingAccounts = [];
    if (withdrawalQueueHeader.tailNode) {
        remainingAccounts.push({
            pubkey: withdrawalQueueHeader.tailNode,
            isWritable: true,
            isSigner: false,
        });
    }

    await program.methods.requestWithdraw(new BN(amount))
        .accounts({
            pool: pool.pubkey,
            poolRedeemTokensMint: pool.redeemMintPubkey,
            withdrawalQueueHeader: pool.withdrawalQueueHeaderPubkey,
            withdrawalQueueNode: withdrawalQueueNewNodeAccountPubkey,
            user: signerKeypair.publicKey,
            userRedeemTokenAccount: signerRedeemTokenAccountPubkey,
        })
        .remainingAccounts(remainingAccounts)
        .signers([signerKeypair])
        .rpc({commitment: "confirmed"});

    return {
        nodePubkey: withdrawalQueueNewNodeAccountPubkey,
        nodeBump: withdrawalQueueNewNodeAccountBump,
    }
}

export async function cancelWithdrawal(program, pool: Pool, signerKeypair: Keypair, withdrawalNode: PublicKey) {
    let withdrawalQueueNode = await program.account.withdrawalQueueNode.fetch(withdrawalNode);
    let remainingAccounts = [];
    if(withdrawalQueueNode.prevNode) {
        remainingAccounts.push({
            pubkey: withdrawalQueueNode.prevNode,
            isWritable: true,
            isSigner: false,
        });
    }

    let signerRedeemTokenAccount = await getAssociatedTokenAddress(pool.redeemMintPubkey, signerKeypair.publicKey);

    await program.methods.cancelWithdrawRequest()
        .accounts({
            pool: pool.pubkey,
            poolRedeemTokensMint: pool.redeemMintPubkey,
            withdrawalQueueHeader: pool.withdrawalQueueHeaderPubkey,
            withdrawalQueueNode: withdrawalNode,
            user: signerKeypair.publicKey,
            userRedeemTokenAccount: signerRedeemTokenAccount,
        })
        .remainingAccounts(remainingAccounts)
        .signers([signerKeypair])
        .rpc({commitment: "confirmed"});
}

export async function processWithdrawalQueue(program, pool: Pool) {
    let poolHeader = await program.account.withdrawalQueueHeader.fetch(pool.withdrawalQueueHeaderPubkey)
    let currentNodeKey = poolHeader.headNode;
    let remaining_accounts = [];

    while(currentNodeKey) {
        let currentNode = await program.account.withdrawalQueueNode.fetch(currentNodeKey);
        let userTokenAccount = await getAssociatedTokenAddress(pool.tokenMintPubkey, currentNode.user);
        remaining_accounts.push({
            pubkey: currentNodeKey,
            isWritable: true,
            isSigner: false,
        });
        remaining_accounts.push({
            pubkey: userTokenAccount,
            isWritable: true,
            isSigner: false,
        });
        currentNodeKey = currentNode.nextNode;
    }

    await program.methods.processWithdrawQueue()
        .accounts({
            pool: pool.pubkey,
            withdrawalQueueHeader: pool.withdrawalQueueHeaderPubkey,
            poolTokenAccount: pool.tokenAccountPubkey,
            tokenMint: pool.tokenMintPubkey,
        })
        .remainingAccounts(remaining_accounts)
        .rpc({commitment: "confirmed"});
}

