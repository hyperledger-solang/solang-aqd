// SPDX-License-Identifier: Apache-2.0

use {anyhow::Result, std::ffi::OsStr, std::process::exit};
use {
    aqd_solana_contracts::{idl_from_json, print_idl_instruction_info},
    aqd_utils::check_target_match,
};

#[derive(Clone, Debug, clap::Args)]
#[clap(
    name = "show",
    about = "Show information about a Solana program's instructions given an IDL JSON file"
)]
pub struct SolanaShow {
    #[clap(long, help = "Specifies the path of the IDL JSON file")]
    idl: String,
    #[clap(
        long,
        help = "Specifies the name of the instruction to show information about\n
                If not specified, information about all instructions is shown"
    )]
    instruction: Option<String>,
    #[clap(long, help = "Specifies whether to export the output in JSON format")]
    output_json: bool,
}

impl SolanaShow {
    /// Handle the Solana show command.
    ///
    /// This function handles the processing of a Solana show command. It checks if the command
    /// is being run in the correct directory, parses the command-line arguments, retrieves the IDL
    /// from a JSON file, and prints information about the instruction.
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
        let instruction = self.instruction.clone();
        let output_json = self.output_json;

        // Get the IDL from the JSON file
        let idl = idl_from_json(OsStr::new(&idl_json))?;

        // Print information about the instruction
        print_idl_instruction_info(&idl, instruction, output_json);

        Ok(())
    }
}
