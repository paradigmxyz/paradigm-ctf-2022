# Paradigm CTF 2022

## Installing

### Prerequisites

* Docker
* [mpwn](https://github.com/lunixbochs/mpwn)
* Python 3

### Configuration

You'll need to set the following environment variables:
* `ETH_RPC_URL` to a valid Ethereum JSON-RPC endpoint
* `PYTHONPATH` to point to mpwn

You'll also need to manually install the following:
* `pip install yaml ecdsa pysha3 web3 cairo-lang`

## Usage

### Build everything

```bash
./build.sh
```

### Run a challenge

Running a challenge will open a port which users will `nc` to. For Ethereum/Starknet related
challenges, an additional port must be supplied so that users can connect to the Ethereum/Starknet
node

```
./run.sh random 31337 8545
```

On another terminal:

```
nc localhost 31337
```

When prompted for the ticket, use any value

```
$ nc localhost 31337
1 - launch new instance
2 - kill instance
3 - get flag
action? 1
ticket please: ticket

your private blockchain has been deployed
it will automatically terminate in 30 minutes
here's some useful information
```

### Running the autosolver

```bash
./solve.sh
```
