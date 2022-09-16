import asyncio
import os
from typing import Awaitable, Callable

from mp import *
from starknet_py.contract import Contract
from starknet_py.net import AccountClient, KeyPair
from starknet_py.net.gateway_client import GatewayClient
from starknet_py.net.models.chains import StarknetChainId
from starknet_py.net.networks import TESTNET
from starkware.crypto.signature.signature import private_to_stark_key
from starkware.starknet.core.os.contract_address.contract_address import \
    calculate_contract_address_from_hash

REMOTE_IP = os.getenv("REMOTE_IP")
REMOTE_PORT = os.getenv("REMOTE_PORT")
HTTP_PORT = os.getenv("HTTP_PORT", "8545")
TICKET = os.getenv("TICKET", "ticket")


async def run_solver_async(
    solver: Callable[[AccountClient, Contract], Awaitable[None]]
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

    player_private_key = int(private, 16)
    player_public_key = private_to_stark_key(player_private_key)

    client = GatewayClient(f"http://{uuid}@{REMOTE_IP}:{HTTP_PORT}", TESTNET)

    # https://github.com/Shard-Labs/starknet-devnet/blob/a5c53a52dcf453603814deedb5091ab8c231c3bd/starknet_devnet/account.py#L35
    player_client = AccountClient(
        client=client,
        address=calculate_contract_address_from_hash(
            salt=20,
            class_hash=1803505466663265559571280894381905521939782500874858933595227108099796801620,
            constructor_calldata=[player_public_key],
            deployer_address=0,
        ),
        key_pair=KeyPair(private_key=player_private_key, public_key=player_public_key),
        chain=StarknetChainId.TESTNET,
    )

    contract = await Contract.from_address(int(setup, 16), player_client)

    await solver(player_client, contract)

    p = remote(REMOTE_IP, int(REMOTE_PORT))

    p >> "action?" << "3\n"

    p >> "ticket please:" << TICKET << "\n"

    output = p.readall().decode("utf8").strip().split("\n")

    if not output[-1].startswith("PCTF"):
        print("failed to get flag")
        raise Exception("\n".join(output))

    print("got flag", output[-1])


def run_solver(solver: Callable[[AccountClient, Contract], Awaitable[None]]):
    asyncio.get_event_loop().run_until_complete(run_solver_async(solver))
