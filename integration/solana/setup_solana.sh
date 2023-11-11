#!/bin/bash
set -e

# Install Solana 
sh -c "$(curl -sSfL https://release.solana.com/v1.17.1/install)"

# Add Solana CLI to PATH
export PATH="$PATH:$HOME/.local/share/solana/install/active_release/bin"

# Ensure nohup writes output to a file, not the terminal.
# If we don't, solana-test-validator might not be able to write its
# output and exit
nohup solana-test-validator -q > validator.out &

# Wait for the validator to start up
sleep 10

# Generate a new keypair
solana-keygen new --no-bip39-passphrase

# Set Solana URL to localhost
solana config set --url localhost

# Air drop 50 SOL (required to send transactions)
solana airdrop 50
