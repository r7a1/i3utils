#!/bin/sh

export CARGO_HOME=/opt/rust
export PATH=$PATH:$CARGO_HOME/bin

cargo $@
