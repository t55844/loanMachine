#!/usr/bin/env bash
set -euo pipefail

# Load env from the Rust server's .env
ENV_FILE="../../../loan_machine_server/.env"
if [ ! -f "$ENV_FILE" ]; then
  echo "Missing $ENV_FILE" >&2; exit 1
fi
set -a
source "$ENV_FILE"
set +a

: "${COOP_REGISTRY_ADDRESS:?must be set in .env}"
: "${START_BLOCK:=0}"

# Render concrete subgraph.yaml from template
envsubst '$COOP_REGISTRY_ADDRESS $START_BLOCK' \
  < subgraph.template.yaml > subgraph.yaml

graph codegen