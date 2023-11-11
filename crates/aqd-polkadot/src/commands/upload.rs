// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::{anyhow, Result},
    colored::Colorize,
    serde_json::{from_str, json, to_string_pretty, Value},
    std::fmt::Debug,
    std::process::exit,
};

use {
    super::CLIExtrinsicOpts,
    aqd_utils::{check_target_match, print_key_value, print_title, print_warning},
    contract_build::Verbosity,
    contract_extrinsics::{ExtrinsicOptsBuilder, UploadCommandBuilder},
};

#[derive(Debug, clap::Args)]
#[clap(name = "upload", about = "Upload a contract on Polkadot")]
pub struct PolkadotUploadCommand {
    #[clap(flatten)]
    extrinsic_cli_opts: CLIExtrinsicOpts,
}

impl PolkadotUploadCommand {
    /// Returns whether to export the call output in JSON format.
    pub fn output_json(&self) -> bool {
        self.extrinsic_cli_opts.output_json
    }

    /// Handles the Polkadot upload command, allowing users to upload contracts to the Polkadot network.
    ///
    /// This function performs the following steps:
    ///
    /// 1. Verifies that the command is being run in the correct directory, failing if it's in a Solang Solana project directory.
    /// 2. Builds command-line options for executing the upload.
    /// 3. Creates and executes the upload command.
    /// 4. Processes the result based on whether execution is required or not.
    /// 5. Prints the outcome, including any generated events or output JSON if requested.
    pub async fn handle(&self) -> Result<()> {
        // Make sure the command is run in the correct directory
        // Fails if the command is run in a Solang Solana project directory
        let target_match = check_target_match("polkadot", None)
            .map_err(|e| anyhow!("Failed to check current directory: {}", e))?;
        if !target_match {
            exit(1);
        }

        // Initialize the extrinsic options
        let cli_options = ExtrinsicOptsBuilder::default()
            .file(Some(self.extrinsic_cli_opts.file.clone()))
            .url(self.extrinsic_cli_opts.url().clone())
            .suri(self.extrinsic_cli_opts.suri.clone())
            .storage_deposit_limit(self.extrinsic_cli_opts.storage_deposit_limit.clone())
            .done();
        let exec = UploadCommandBuilder::default()
            .extrinsic_opts(cli_options)
            .done()
            .await?;

        // Obtain the code hash
        // This is used to check if the contract has already been uploaded
        let code_hash = exec.code().code_hash();

        if !self.extrinsic_cli_opts.execute {
            let result = exec
                .upload_code_rpc()
                .await?
                .map_err(|err| anyhow!("Error uploading the code: {:?}", err))?;
            if self.output_json() {
                let json_object = json!({
                    "result": "Success",
                    "code_hash": result.code_hash,
                    "deposit": result.deposit
                });
                println!("{}", to_string_pretty(&json_object)?);
            } else {
                print_title!("Upload Dry Run Result");
                print_key_value!("Status", "Success");
                print_key_value!("Code hash", format!("{:?}", result.code_hash));
                print_key_value!("Deposit", format!("{:?}", result.deposit));
                print_warning!("Execution of your upload call has NOT been completed. To submit the transaction and execute the call on chain, please include -x/--execute flag.");
            }
        } else {
            let result = exec
                .upload_code()
                .await
                .map_err(|err| anyhow!("Error uploading the code: {}", err.to_string()))?;
            let events = result.display_events;
            let events = if self.output_json() {
                events.to_json()?
            } else {
                events.display_events(Verbosity::Default, exec.token_metadata())?
            };
            let code_stored = result.code_stored.ok_or_else(|| {
                anyhow!(
                    "This contract has already been uploaded. Code hash: 0x{}",
                    hex::encode(code_hash)
                )
            })?;
            if self.output_json() {
                let json_object = json!({
                    "events": from_str::<Value>(&events)?,
                    "code_hash": code_stored.code_hash,
                });
                println!("{}", to_string_pretty(&json_object)?);
            } else {
                println!("{}", events);
                print_key_value!("Code hash", format!("{:?}", code_stored.code_hash));
            }
        }
        Ok(())
    }
}
