#!/bin/sh

# Performs `cargo build` and prints the path of the compiled executable to stdout #

json=$(cargo build --message-format=json-render-diagnostics "$@") && \
printf "%s" "$json" | jq -js '[.[] | select(.reason == "compiler-artifact") | select(.executable != null)] | last | .executable'
