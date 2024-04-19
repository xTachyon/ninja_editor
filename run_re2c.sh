#!/bin/bash

set -e 

re2rust -i --no-generation-date --no-version -o ninja_editor/src/lexer.rs ninja_editor/src/lexer.in.rs
rustfmt ninja_editor/src/lexer.rs