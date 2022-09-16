#!/bin/bash

IMAGE="gcr.io/paradigm-ctf/2022/$1:latest"
PORT="$2"
HTTP_PORT="$3"

echo "[+] running challenge"
exec docker run \
    -e "PORT=$PORT" \
    -e "HTTP_PORT=$HTTP_PORT" \
    -e "ETH_RPC_URL=$ETH_RPC_URL" \
    -e "FLAG=PCTF{flag}" \
    -p "$PORT:$PORT" \
    -p "$HTTP_PORT:$HTTP_PORT" \
    "$IMAGE"