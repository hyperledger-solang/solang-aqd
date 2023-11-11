// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::{anyhow, Result},
    colored::Colorize,
    std::fmt::Debug,
    std::process::exit,
};

use {
    super::CLIExtrinsicOpts,
    aqd_utils::{
        check_target_match, print_key_value, print_title, print_warning, prompt_confirm_transaction,
    },
    contract_build::{util::decode_hex, Verbosity},
    contract_extrinsics::{
        BalanceVariant, DisplayEvents, ExtrinsicOptsBuilder, InstantiateCommandBuilder,
    },
    sp_core::Bytes,
};

#[derive(Debug, clap::Args)]
#[clap(name = "instantiate", about = "Instantiate a contract on Polkadot")]
pub struct PolkadotInstantiateCommand {
    #[clap(
        name = "constructor",
        long,
        default_value = "new",
        help = "Specifies the name of the contract constructor to call."
    )]
    constructor: String,
    #[clap(long, num_args = 0.., help = "Specifies the arguments of the contract constructor to call.")]
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
    #[clap(
        long,
        help = "Specifies the maximum proof size for this instantiation."
    )]
    proof_size: Option<u64>,
    #[clap(long, value_parser = parse_hex_bytes, help = "Specifies a salt used in the address derivation of the new contract.")]
    salt: Option<Bytes>,
    #[clap(
        short('y'),
        long,
        help = "Specifies whether to skip the confirmation prompt."
    )]
    skip_confirm: bool,
}

/// Parse hex encoded bytes.
fn parse_hex_bytes(input: &str) -> Result<Bytes> {
    let bytes = decode_hex(input)?;
    Ok(bytes.into())
}

impl PolkadotInstantiateCommand {
    /// Returns whether to export the call output in JSON format.
    pub fn output_json(&self) -> bool {
        self.extrinsic_cli_opts.output_json
    }

    /// Handles the instantiation of a contract on the Polkadot network.
    ///
    /// If the `execute` flag is set to `false`, it performs a dry run of the instantiation and displays
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
        let exec = InstantiateCommandBuilder::default()
            .constructor(self.constructor.clone())
            .args(self.args.clone())
            .extrinsic_opts(cli_options)
            .value(self.value.clone())
            .gas_limit(self.gas_limit)
            .proof_size(self.proof_size)
            .salt(self.salt.clone())
            .done()
            .await?;

        if !self.extrinsic_cli_opts.execute {
            let result = exec.instantiate_dry_run().await?;
            let dry_run_result = exec
                .decode_instantiate_dry_run(&result)
                .await
                .map_err(|e| anyhow!("Failed to decode instantiate dry run result: {}", e))?;
            if self.output_json() {
                println!("{}", dry_run_result.to_json()?);
            } else {
                print_title!("Instantiate dry run result");
                print_key_value!("Status", format!("{}", &dry_run_result.result));
                print_key_value!("Reverted", format!("{:?}", &dry_run_result.reverted));
                print_key_value!("Contract", &dry_run_result.contract);
                print_key_value!("Gas consumed", &dry_run_result.gas_consumed.to_string());
                print_warning!("Execution of your instantiate call has NOT been completed. To submit the transaction and execute the call on chain, please include -x/--execute flag.");
            }
        } else {
            let gas_limit = exec.estimate_gas().await?;
            if !self.skip_confirm {
                prompt_confirm_transaction(|| {
                    println!("Instantiation Summary:");
                    print_key_value!("Constructor", exec.args().constructor());
                    print_key_value!("Args", exec.args().raw_args().join(" "));
                    print_key_value!("Gas limit", gas_limit.to_string());
                })?;
            }
            let instantiate_result = exec
                .instantiate(Some(gas_limit))
                .await
                .map_err(|err| anyhow!("Error instantiating the contract: {:?}", err))?;
            let events = DisplayEvents::from_events(
                &instantiate_result.result,
                Some(exec.transcoder()),
                &exec.client().metadata(),
            )?;
            let contract_address = instantiate_result.contract_address.to_string();
            if self.output_json() {
                let display_instantiate_result = InstantiateResult {
                    code_hash: instantiate_result.code_hash.map(|ch| format!("{ch:?}")),
                    contract: contract_address,
                    events,
                };
                println!("{}", display_instantiate_result.to_json()?)
            } else {
                println!(
                    "{}",
                    events
                        .display_events(Verbosity::Default, &instantiate_result.token_metadata)?
                );
                if let Some(code_hash) = instantiate_result.code_hash {
                    print_key_value!("Code hash", format!("{code_hash:?}"));
                }
                print_key_value!("Contract", contract_address);
            };
        }
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct InstantiateResult {
    pub contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_hash: Option<String>,
    pub events: DisplayEvents,
}

impl InstantiateResult {
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
