#!/bin/bash
set -euxo pipefail

daemonize /ctf/docker/solana-test-validator-1.9.29

sleep 5;

daemonize -ao /ctf/validator.log /ctf/docker/solana-1.9.29 logs -u l
daemonize -ao /ctf/server.log -e /ctf/error.log -c /ctf/server /ctf/server/target/release/server

sleep infinity;
