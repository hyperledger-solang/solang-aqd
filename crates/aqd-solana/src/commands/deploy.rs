// SPDX-License-Identifier: Apache-2.0

use {anyhow::Result, serde_json::json, std::process::exit};
use {aqd_solana_contracts::deploy_program, aqd_utils::check_target_match};

#[derive(Clone, Debug, clap::Args)]
#[clap(name = "deploy", about = "Deploy a program to Solana")]
pub struct SolanaDeploy {
    #[clap(help = "Specifies the path to the program file to deploy (.so)")]
    program_location: String,
    #[clap(long, help = "Specifies whether to export the output in JSON format")]
    output_json: bool,
}

impl SolanaDeploy {
    /// Handle the deployment of a Solana program
    ///
    /// This function is responsible for managing the deployment process,
    /// including checking the current directory, parsing command-line arguments,
    /// configuring settings, and executing the deployment command. It also handles
    /// loading the necessary configuration and signers, defining output formats,
    /// and processing the deployment command using the provided configuration.
    pub fn handle(&self) -> Result<()> {
        // Make sure the command is run in the correct directory
        // Fails if the command is run in a Solang Polkadot project directory
        let target_match = check_target_match("solana", None)
            .map_err(|e| anyhow::anyhow!("Failed to check current directory: {}", e))?;
        if !target_match {
            exit(1);
        }

        // Parse command-line arguments
        let program_location = self.program_location.clone();
        let output_json = self.output_json;

        // Deploy the program
        let program_id = deploy_program(program_location)?;

        // If the output is JSON, print the program ID in JSON format
        // Else, print the program ID as a string
        if output_json {
            let program_id = json!({ "program_id": program_id });
            println!("{}", program_id);
        } else {
            println!("Program ID: {}", program_id);
        }

        Ok(())
    }
}
