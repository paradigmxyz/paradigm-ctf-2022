from paradigmctf.eth_challenge import *

from web3 import Web3
from eth_keys import keys
from eth_account import Account
from eth_account.signers.local import LocalAccount
from random import randint

# Need to find some a private key corresponding to a public key that is also valid bytecode. An easy way to do this is to find one with 
# a public key that has a zero byte at the start (since this is the STOP instruction). 
def find_account() -> LocalAccount:
    while True:
        i = randint(0, 2**256-1)
        priv = i.to_bytes(32, byteorder="little")
        pub = keys.PrivateKey(priv).public_key
        if pub.to_bytes()[0] == 0:
            return Account.from_key(priv), pub.to_hex()[2:]

account, account_pubkey = find_account()

def solve(web3: Web3, player: Account, setup: str):
    
    send_tx(web3, {
        "from": player.address,
        "to": account.address,
        "value": web3.toWei(100, "ether"),
    })

    # get address of lockbox2 contract
    lockbox2Addr = web3.eth.call(
            {
                "from": account.address,
                "to": setup,
                "data": web3.sha3(text="lockbox2()")[:4].hex(),
            }
        )[-20:]

    data =  "0x890d6908"                                                                # solve() function selector
    data += "0000000000000000000000000000000000000000000000000000000000000061"          # 97 (prime number)
    data += "00000000000000000000000000000000000000000000000000000000000001af"          # 431 (prime number)
    data += "0000000000000000000000000000000000000000000000000000000000000001"          # 1
    data += "000000000000000000000000000000000000000000000000000000000000000100"        # 1 (followed by two zeroes so that bytes in stage 4 isn't too long)
    data += f"7f{account_pubkey[:64]}6000527f{account_pubkey[64:]}60205260406000f3"     # initcode to create contract with pub_key as bytecode
    data += "0" * (998 - len(data))                                                     # fill up the rest of the remaining space with 0

    sign_send_tx(web3, account, {
        "to": lockbox2Addr,
        "gas": 430000,              # Carefully chosen gas amount, enough for inner call to OOG but outer call still has enough
        "gasPrice": 0,
        "nonce": 0,
        "data": data
    })


run_solver(solve)