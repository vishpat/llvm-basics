#!/bin/bash

rm -f main main.ll
export RUST_LOG=debug
set -x
cargo run 
lli main.ll
echo $?
