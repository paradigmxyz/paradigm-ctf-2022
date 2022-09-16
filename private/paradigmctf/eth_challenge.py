from mp import *

import json
import subprocess
import os
import time
import binascii

from web3 import Web3
from web3.exceptions import TransactionNotFound
from eth_account import Account
from eth_abi import encode

REMOTE_IP = os.getenv("REMOTE_IP")
REMOTE_PORT = os.getenv("REMOTE_PORT")
DEPLOY_ETH = os.getenv("DEPLOY_ETH", "0")
TICKET = os.getenv("TICKET", "ticket")

def compile(runtime=False) -> str:
    cwd = os.getcwd()
    parent = os.path.dirname(cwd)
    
    result = subprocess.run(
        ["env", "solc", f"public={parent}/public/contracts/", f"private={parent}/private/", "--combined-json", "bin,bin-runtime", "Exploit.sol"],
        capture_output=True,
        env={
            **os.environ,
        },
    )
    if result.returncode:
        print(result.stdout)
        print(result.stderr)
        raise Exception(result.stderr.decode('utf8'))

    compiled = json.loads(result.stdout)
    return binascii.unhexlify(compiled['contracts']['Exploit.sol:Exploit']['bin-runtime' if runtime else 'bin'])


def encode_function_call(web3, functionName, types, args):
    selector = web3.sha3(text=f"{functionName}({','.join(types)})")[:4].hex()

    return selector + encode(types, args).hex()

def send_tx(web3, tx):
    txhash = web3.eth.sendTransaction(tx)

    while True:
        try:
            rcpt = web3.eth.getTransactionReceipt(txhash)
            break
        except TransactionNotFound:
            time.sleep(0.1)

    if rcpt.status != 1:
        raise Exception("deployment failed")
    return rcpt

def send_txs(web3, txs):
    txhashes = [web3.eth.sendTransaction(tx) for tx in txs]
    
    for txhash in txhashes:
        while True:
            try:
                rcpt = web3.eth.getTransactionReceipt(txhash)
                break
            except TransactionNotFound:
                time.sleep(0.1)

        if rcpt.status != 1:
            raise Exception("deployment failed")

def sign_send_tx(web3, account, tx):
    raw = account.sign_transaction(tx)
    txhash = web3.eth.sendRawTransaction(raw.rawTransaction)

    while True:
        try:
            rcpt = web3.eth.getTransactionReceipt(txhash)
            break
        except TransactionNotFound:
            time.sleep(0.1)
    
    if rcpt.status != 1:
        raise Exception("deployment failed")
    return rcpt


def run_solver(
    solver: Callable[[Web3, str, str], None]
):
    p = remote(REMOTE_IP, int(REMOTE_PORT))

    p >> "action?" << "1\n"

    p >> "ticket please:" << TICKET << "\n"

    p >> "uuid:"
    uuid = p.recvline().strip().decode("utf8")

    p >> "rpc endpoint:"
    rpc = p.recvline().strip().decode("utf8")

    p >> "private key:"
    private = p.recvline().strip().decode("utf8")

    p >> "contract:"
    setup = p.recvline().strip().decode("utf8")

    web3 = Web3(Web3.HTTPProvider(rpc))

    account = Account.from_key(private)

    solver(web3, account, setup)

    p = remote(REMOTE_IP, int(REMOTE_PORT))

    p >> "action?" << "3\n"

    p >> "ticket please:" << TICKET << "\n"

    output = p.readall().decode("utf8").strip().split("\n")

    if not output[-1].startswith("PCTF"):
        print("failed to get flag")
        raise Exception("\n".join(output))

    print("got flag", output[-1])


def solve_with_exploit(web3, player, setup):
    code = compile()
    
    send_tx(web3, {
        "from": player.address,
        "gas": 12_500_000,
        "data": code + encode(['address'], [setup]),
        "value": Web3.toWei(DEPLOY_ETH, "ether"),
    })

if __name__ == "__main__":
    run_solver(solve_with_exploit)