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
    aqd_utils::{check_target_match, print_key_value},
    contract_build::Verbosity,
    contract_extrinsics::{
        parse_code_hash, DefaultConfig, ExtrinsicOptsBuilder, RemoveCommandBuilder,
    },
    subxt::Config,
};

#[derive(Debug, clap::Args)]
#[clap(name = "remove", about = "Remove a contract on Polkadot")]
pub struct PolkadotRemoveCommand {
    #[clap(long, value_parser = parse_code_hash, help = "Specifies the code hash to remove.")]
    code_hash: Option<<DefaultConfig as Config>::Hash>,
    #[clap(flatten)]
    extrinsic_cli_opts: CLIExtrinsicOpts,
}

impl PolkadotRemoveCommand {
    /// Returns whether to export the call output in JSON format.
    pub fn output_json(&self) -> bool {
        self.extrinsic_cli_opts.output_json
    }

    /// Handles the removal of a contract from the Polkadot network.
    ///
    /// Removes a contract with the specified code hash. If successful, it returns information about the
    /// removal, including the events generated. The `output_json` flag controls the output format.
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
        let exec = RemoveCommandBuilder::default()
            .code_hash(self.code_hash)
            .extrinsic_opts(cli_options)
            .done()
            .await?;

        let remove_result = exec
            .remove_code()
            .await
            .map_err(|err| anyhow!("Error removing the code: {}", err.to_string()))?;
        let display_events = remove_result.display_events;
        let events = if self.output_json() {
            display_events.to_json()?
        } else {
            display_events.display_events(Verbosity::Default, exec.token_metadata())?
        };
        let code_removed = remove_result.code_removed.ok_or_else(|| {
            anyhow!(
                "Error removing the code: {}",
                hex::encode(exec.final_code_hash())
            )
        })?;
        let remove_result = code_removed.code_hash;
        if self.output_json() {
            let json_object = json!({
                "events": from_str::<Value>(&events)?,
                "removed_code_hash": remove_result,
            });
            let json_object = to_string_pretty(&json_object)?;
            println!("{}", json_object);
        } else {
            println!("{events}");
            print_key_value!("Code hash", format!("{remove_result:?}"));
        }
        Ok(())
    }
}
