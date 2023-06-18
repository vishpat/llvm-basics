#!/bin/bash

rm -f main main.ll
export RUST_LOG=debug
set -x
cargo run --bin $1 
clang -o main main.ll
./main
echo $?
