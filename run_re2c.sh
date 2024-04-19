#!/bin/bash

set -e 

re2rust -i --no-generation-date --no-version -o src/lexer.rs src/lexer.in.rs
rustfmt src/lexer.rs