# Solana Contracts

`aqd-solana-contracts` is a Rust crate designed to simplify interaction with smart contracts on the Solana blockchain.
This crate provides a set of utilities and abstractions to make it easier to deploy smart contracts and to call specific functions on Solana smart contracts, leveraging the contract's Instruction Description Language (IDL).


## Example usage
The following example demonstrates how to use `aqd-solana-contracts` to deploy a smart contract to the Solana blockchain. 
```rust
use {
    anyhow::Result,
    aqd_solana_contracts::deploy_program,
};

fn main() -> Result<()> {
    // Define the contract to call
    let program_path = "flipper.so".to_string();

    // Deploy the contract (This returns the program ID)
    let program_id = deploy_program(program_path)?;

    Ok(())
}
```



The following example demonstrates how to use `aqd-solana-contracts` to call a method of a deployed smart contract on the Solana blockchain.
```rust
use {
    anyhow::Result,
    solana_clap_v3_utils::input_validators::normalize_to_url_if_moniker,
    solana_cli_config::{Config, CONFIG_FILE},
    aqd_solana_contracts::{print_transaction_information, SolanaTransaction},
};

fn main() -> Result<()> {
    // Parse the config file to get the RPC URL and payer keypair.
    let config_file = CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Error loading config file"))?;
    let cli_config = Config::load(config_file).unwrap_or_default();
    let rpc_url = normalize_to_url_if_moniker(&cli_config.json_rpc_url);
    let keypair = cli_config.keypair_path.to_string();

    // Define the path to the IDL JSON file, the program ID, and whether to output JSON.
    let idl_json = "crates/aqd-solana-contracts/examples/contracts/flipper.json";
    // The program ID is the address of the deployed program on the Solana blockchain.
    // Replace this with the address of the deployed flipper program.
    let program_id = "71gxeC5D6bGAUznocUWyXdhWQozhDc72qKL7oZ8zn4kR";
    let output_json = false;

    // Call the `new` method of the flipper program.

    // Define the instruction name, data arguments, and accounts arguments.
    let instruction_name = "new";
    let data_args: Vec<String> = vec!["true".to_string()];
    let accounts_args: Vec<String> =
        vec!["new".to_string(), "self".to_string(), "system".to_string()];

    // Create a `SolanaTransaction` object with the necessary parameters.
    let flipper_new = SolanaTransaction::new()
        .rpc_url(rpc_url.clone())
        .idl(idl_json.to_string())
        .program_id(program_id.to_string())
        .instruction(instruction_name.to_string())
        .call_data(data_args)
        .accounts(accounts_args)
        .payer(keypair.clone())
        .done()?;

    // Submit the transaction.
    let signature = flipper_new.submit_transaction()?;

    // Print the transaction information.
    match print_transaction_information(
        flipper_new.rpc_client(),
        &signature,
        flipper_new.instruction(),
        flipper_new.idl().types.as_slice(),
        flipper_new.new_accounts(),
        output_json,
    ) {
        Ok(_) => (),
        Err(err) => eprintln!("{}", err),
    }

    Ok(())
}
```
In this example, `aqd-solana-contracts` streamlines the process of interacting with a Solana smart contract. It takes care of data encoding and prepares the necessary accounts, allowing you to focus on defining the specifics of the transaction and easily submit it to the Solana blockchain.

> For more examples, see the [examples](examples) directory.
