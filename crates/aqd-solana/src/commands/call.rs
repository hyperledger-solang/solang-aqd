// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::Result,
    solana_clap_v3_utils::input_validators::normalize_to_url_if_moniker,
    solana_cli_config::{Config, CONFIG_FILE},
    std::process::exit,
};
use {
    aqd_solana_contracts::{print_transaction_information, SolanaTransaction},
    aqd_utils::check_target_match,
};

#[derive(Clone, Debug, clap::Args)]
#[clap(name = "call", about = "Send a custom transaction to a Solana program")]
pub struct SolanaCall {
    #[clap(long, help = "Specifies the path of the IDL JSON file")]
    idl: String,
    #[clap(long, help = "Specifies the program ID of the deployed program")]
    program: String,
    #[clap(long, help = "Specifies the name of the instruction to call")]
    instruction: String,
    #[clap(
        long,
        help = "Specifies the data arguments to pass to the instruction.
                For arrays and vectors, pass a comma-separated list of values. (e.g. 1,2,3,4)
                For structs, pass a JSON string of the struct. (can be a path to a JSON file)",
        // The number of data arguments is variable (Can be 0 or more)
        num_args = 0..,
    )]
    data: Vec<String>,
    #[clap(
        long,
        help = "Specifies the accounts arguments to pass to the instruction\
        Keywords:
        - new: create a new account
        - self: reads the default keypair from the local configuration file.
        - system: use the system program ID as the account",
        // The number of accounts arguments is variable (Can be 0 or more)
        num_args = 0..,
    )]
    accounts: Vec<String>,
    #[clap(long, help = "Specifies the payer keypair to use for the transaction")]
    payer: Option<String>,
    #[clap(long, help = "Specifies whether to export the output in JSON format")]
    output_json: bool,
}

impl SolanaCall {
    /// Handle the Solana transaction command.
    ///
    /// This function handles the processing of a Solana transaction command. It checks if the command
    /// is being run in the correct directory, parses the command-line arguments, retrieves the RPC URL
    /// and payer keypair from the configuration file, creates a [`SolanaTransaction`] object, submits
    /// the transaction, and prints transaction information.
    pub fn handle(&self) -> Result<()> {
        // Make sure the command is run in the correct directory
        // Fails if the command is run in a Solang Polkadot project directory
        let target_match = check_target_match("solana", None)
            .map_err(|e| anyhow::anyhow!("Failed to check current directory: {}", e))?;
        if !target_match {
            exit(1);
        }

        // Parse command-line arguments
        let idl_json = self.idl.clone();
        let program_id = self.program.clone();
        let instruction = self.instruction.clone();
        let data_args = self.data.clone();
        let accounts_args = self.accounts.clone();
        let payer = self.payer.clone();
        let output_json = self.output_json;

        // Get the RPC URL from the config file
        // Parse the config file to get the RPC URL and payer keypair.
        let config_file = CONFIG_FILE
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Error loading config file"))?;
        let cli_config = Config::load(config_file).unwrap_or_default();
        let rpc_url = normalize_to_url_if_moniker(&cli_config.json_rpc_url);
        let keypair = cli_config.keypair_path.to_string();

        let payer = payer.unwrap_or(keypair);

        // Create a `SolanaTransaction` object with the necessary parameters.
        let transaction = SolanaTransaction::new()
            .rpc_url(rpc_url.clone())
            .idl(idl_json.to_string())
            .program_id(program_id.to_string())
            .instruction(instruction.to_string())
            .call_data(data_args)
            .accounts(accounts_args)
            .payer(payer.clone())
            .done()?;

        // Submit the transaction.
        let signature = transaction.submit_transaction()?;

        // Print the transaction information.
        print_transaction_information(
            transaction.rpc_client(),
            &signature,
            transaction.instruction(),
            transaction.idl().types.as_slice(),
            transaction.new_accounts(),
            output_json,
        )
    }
}
