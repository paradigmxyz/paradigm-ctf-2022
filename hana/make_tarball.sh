#!/bin/bash
set -euxo pipefail

rm -rf solhana-ctf
mkdir -p solhana-ctf

# prebuilt binaries, idls, dockerfile, readme
cp -r elf solhana-ctf/
cp -r idl solhana-ctf/
cp Dockerfile solhana-ctf/
cp README.md solhana-ctf/

# code for challenge problems
cp -r chain solhana-ctf/
rm -rf solhana-ctf/chain/target

# client code/skeletons for solutions
cp -r client solhana-ctf/
rm -rf solhana-ctf/client/node_modules

# server code/binary for setting up chain
cp -r server solhana-ctf/
rm -rf solhana-ctf/server/target
rm -f solhana-ctf/server/hana-ctf.db

# keys. we gen a new master key for obvious reason
# assuming i didnt fuck it up, giving the player all the rest is fine
# authority of everything is either master or random
cp -r keys solhana-ctf/
solana-keygen new -f -s --no-bip39-passphrase -o solhana-ctf/keys/master.json

# that should be everything!
tar czf solhana-ctf.tar.gz solhana-ctf
rm -rf solhana-ctf
