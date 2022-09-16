#!/bin/bash

set -o errexit

pushd framework
cargo clean
popd

pushd framework/chall
cargo clean
popd

pushd framework-solve
cargo clean
popd

pushd framework-solve/solve
cargo clean
popd
