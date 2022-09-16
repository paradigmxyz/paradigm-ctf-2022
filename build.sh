#!/bin/bash

set -euo pipefail

build_challenge() {
    name="$1"
    echo "building $name"

    tag="gcr.io/paradigm-ctf/2022/$name:latest"

    docker buildx build --platform linux/amd64 -t "$tag" "$name/public"
}

declare -a chals=(
    "stealing-sats"
    "vanity"
    "merkledrop"
    "sourcecode"
    "cairo-proxy"
    "riddle-of-the-sphinx"
    "rescue"
    "random"
    "electric-sheep"
    "trapdooor"
    "trapdoooor"
    "cairo-auction"
    "hint-finance"
    "fun-reversing-challenge"
    "just-in-time"
    "lockbox2"
)

for chal in "${chals[@]}"; do
    build_challenge $chal
done
