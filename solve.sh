#!/bin/bash

set -euo pipefail

function read_key() {
    python3 -c 'import sys; import yaml; print(yaml.safe_load(open(sys.argv[1]))[sys.argv[2]])' "$1" "$2"
}

function solve_one() {
    CHAL="$1"
    VERSION="${2:-}"
    ETH="${3:-0}"
    REMOTE_IP="127.0.0.1"
    REMOTE_PORT="31337"

    echo "[+] solving $CHAL"

    ./run.sh "$CHAL" "31337" "$(read_key "$CHAL/info.yaml" rpc_port)" >/dev/null 2>&1 &
    CHAL_PID="$!"
    function cleanup()
    {
        kill "$CHAL_PID"
    }

    trap cleanup EXIT

    sleep 2

    pushd "$CHAL/private" >/dev/null

    if [ -z "$VERSION" ]; then
        REMOTE_IP="$REMOTE_IP" REMOTE_PORT="$REMOTE_PORT" python3 solve.py
    else
        if [ -f "solve.py" ]; then
            file="solve.py"
        else
            file="../../private/paradigmctf/eth_challenge.py"
        fi
        PYTHONDONTWRITEBYTECODE=1 PYTHONPATH="$PYTHONPATH:../../private" SOLC_VERSION=$VERSION REMOTE_IP="$REMOTE_IP" REMOTE_PORT="$REMOTE_PORT" DEPLOY_ETH=$ETH HTTP_PORT="8545" python3 $file
    fi

    popd >/dev/null

    kill "$CHAL_PID"
}

declare -a chals=(
    # "stealing-sats 0.8.16 1"
    # "vanity 0.7.6"
    # "merkledrop 0.8.16"
    # "lockbox2 0.8.16"
    # "hint-finance 0.8.16 50"
    # "sourcecode 0.8.16"
    # "cairo-proxy 0.0.0"
    # "cairo-auction 0.0.0"
    # "riddle-of-the-sphinx 0.0.0"
    # "just-in-time 0.8.16"
    # "rescue 0.8.16 25"
    # "random 0.8.16"
    "trapdooor"
    "trapdoooor"
)

for chal in "${chals[@]}"; do
    solve_one $chal
done
