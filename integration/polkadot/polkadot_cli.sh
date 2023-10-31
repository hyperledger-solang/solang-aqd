#!/bin/bash
set -e

# Add Aqd to PATH (assumes aqd is built)
# pwd is integration/polkadot, so ../../target/release
export PATH="$PATH:$(pwd)/../../target/release"


# Test case 1:
# 1. Upload "flipper" contract
# 2. Instantiate contract
# 3. Call get contract (should return true)
# 4. Call flip contract 
# 5. Call get contract (should return false)

# Upload the contract to a substrate node
upload_result=$(aqd polkadot upload --suri //Alice -x flipper.contract --output-json)

# Instantiate the contract
instantiate_result=$(aqd polkadot instantiate --suri //Alice --args true -x flipper.contract --output-json --skip-confirm)

# Extract the contract address from the instantiate result
contract_address=$(echo "$instantiate_result" | jq -r '.contract')

# Call the contract
call_result=$(aqd polkadot call --contract "$contract_address" --message get --suri //Alice flipper.contract --output-json --skip-confirm)

# Extract the "value" field from the "data" object
value=$(echo "$call_result" | jq -r '.data.Tuple.values[0].Bool')

# Step 5: Assert that "value" is true
if [ "$value" == "true" ]; then
    echo "Contract call succeeded."
else
    echo "Contract call reverted."
    exit 1
fi

# Call the flip function on the contract
call_flip_result=$(aqd polkadot call --contract "$contract_address" --message flip --suri //Alice -x flipper.contract --output-json --skip-confirm)

# Check that "value" is now false
call_result=$(aqd polkadot call --contract "$contract_address" --message get --suri //Alice flipper.contract --output-json --skip-confirm)

# Extract the "value" field from the "data" object
value=$(echo "$call_result" | jq -r '.data.Tuple.values[0].Bool')

# Assert that "value" is false
if [ "$value" == "false" ]; then
    echo "Contract call flipped as expected."
else
    echo "Contract call did not flip the value."
    exit 1
fi


# Test case 2:
# 1. Upload "incrementer" contract
# 2. Remove contract using the code hash

# Upload the contract to a substrate node
upload_result=$(aqd polkadot upload --suri //Alice -x incrementer.contract --output-json)

# Extract the "code_hash" value directly from the JSON object
code_hash=$(echo "$upload_result" | jq -r '.code_hash')

# Remove the contract using the code hash
remove_result=$(aqd polkadot remove --suri //Alice --output-json --code-hash "$code_hash" incrementer.contract)

# Get  "removed_code_hash" value from the JSON object
removed_code_hash=$(echo "$remove_result" | jq -r '.removed_code_hash')

# In case of success, the removed_code_hash should be equal to the code_hash
if [ "$removed_code_hash" == "$code_hash" ]; then
    echo "Contract removed successfully."
else
    echo "Contract removal failed."
    exit 1
fi
