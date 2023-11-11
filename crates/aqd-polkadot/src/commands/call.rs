// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::{anyhow, Context, Result},
    colored::Colorize,
    serde_json::{json, to_string_pretty},
    std::{fmt::Debug, process::exit},
};

use {
    super::CLIExtrinsicOpts,
    aqd_utils::{
        check_target_match, print_key_value, print_title, print_warning, prompt_confirm_transaction,
    },
    contract_build::Verbosity,
    contract_extrinsics::{
        BalanceVariant, CallCommandBuilder, DefaultConfig, ExtrinsicOptsBuilder, StorageDeposit,
    },
    subxt::Config,
};

#[derive(Debug, clap::Args)]
#[clap(name = "call", about = "Call a contract on Polkadot")]
pub struct PolkadotCallCommand {
    #[clap(
        name = "contract",
        long,
        help = "Specifies the address of the contract to call."
    )]
    contract: <DefaultConfig as Config>::AccountId,
    #[clap(
        long,
        short,
        help = "Specifies the name of the contract message to call."
    )]
    message: String,
    #[clap(long, num_args = 0.., help = "Specifies the arguments of the contract message to call.")]
    args: Vec<String>,
    #[clap(flatten)]
    extrinsic_cli_opts: CLIExtrinsicOpts,
    #[clap(
        name = "value",
        long,
        default_value = "0",
        help = "Specifies the value to be transferred as part of the call."
    )]
    value: BalanceVariant,
    #[clap(
        name = "gas",
        long,
        help = "Specifies the maximum amount of gas to be used for this command."
    )]
    gas_limit: Option<u64>,
    #[clap(long, help = "Specifies the maximum proof size for this call.")]
    proof_size: Option<u64>,
    #[clap(
        short('y'),
        long,
        help = "Specifies whether to skip the confirmation prompt."
    )]
    skip_confirm: bool,
}

impl PolkadotCallCommand {
    /// Returns whether to export the call output in JSON format.
    pub fn output_json(&self) -> bool {
        self.extrinsic_cli_opts.output_json
    }

    /// Handles the calling of a contract on the Polkadot network.
    ///
    /// If the `execute` flag is set to `false`, it performs a dry run of the call and displays
    /// the results. If the `output_json` flag is set to `true`, the output is in JSON format.
    /// Otherwise, it prompts for a transaction confirmation and then submits the transaction for execution.
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
        let exec = CallCommandBuilder::default()
            .contract(self.contract.clone())
            .message(self.message.clone())
            .args(self.args.clone())
            .extrinsic_opts(cli_options)
            .gas_limit(self.gas_limit)
            .proof_size(self.proof_size)
            .value(self.value.clone())
            .done()
            .await?;

        if !self.extrinsic_cli_opts.execute {
            let result = exec.call_dry_run().await?;
            let ret_val = &result
                .result
                .map_err(|err| anyhow!("Error calling the contract: {:?}", err))?;
            let value = exec
                .transcoder()
                .decode_message_return(exec.message(), &mut &ret_val.data[..])
                .context(format!("Failed to decode return value {:?}", &ret_val))?;
            if self.output_json() {
                let json_object = json!({
                    "reverted": ret_val.did_revert(),
                    "data": value,
                    "gas_consumed": result.gas_consumed,
                    "gas_required": result.gas_required,
                    "storage_deposit": StorageDeposit::from(&result.storage_deposit),
                });
                println!("{}", to_string_pretty(&json_object)?);
            } else {
                print_title!("Call Dry Run Result");
                print_key_value!("Status", format!("{}", value));
                print_key_value!("Reverted", format!("{:?}", ret_val.did_revert()));
                print_warning!("Execution of your call has NOT been completed. To submit the transaction and execute the call on chain, please include -x/--execute flag.");
            };
        } else {
            let gas_limit = exec.estimate_gas().await?;
            if !self.skip_confirm {
                prompt_confirm_transaction(|| {
                    println!("Call Summary:");
                    print_key_value!("Message", exec.message());
                    print_key_value!("Args", exec.args().join(" "));
                    print_key_value!("Gas limit", gas_limit.to_string());
                })?;
            }
            let token_metadata = exec.token_metadata();
            let display_events = exec
                .call(Some(gas_limit))
                .await
                .map_err(|err| anyhow!("Error calling the contract: {:?}", err))?;
            let output = if self.output_json() {
                display_events.to_json()?
            } else {
                display_events.display_events(Verbosity::Default, token_metadata)?
            };
            println!("{output}");
        }
        Ok(())
    }
}
