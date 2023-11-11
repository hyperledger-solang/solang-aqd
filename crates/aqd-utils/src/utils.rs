// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::{anyhow, Result},
    std::{
        fs::File,
        io,
        io::{Read, Write},
        path::PathBuf,
    },
};

/// Prompt the user to confirm transaction.
pub fn prompt_confirm_transaction<F: FnOnce()>(summary: F) -> Result<()> {
    summary();
    println!("Are you sure you want to submit this transaction? (Y/n): ");

    let mut choice = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut choice)?;
    match choice.trim().to_lowercase().as_str() {
        "y" | "" => Ok(()),
        "n" => Err(anyhow!("Transaction not submitted")),
        _ => Err(anyhow!("Invalid choice")),
    }
}

/// A helper function to check if the target name provided by the user matches the target name in solang.toml
///
/// If no configuration file content is provided, then the function will read the content of the
/// solang.toml file in the current directory.
///
/// If the target names match, then the function will return true. else, it will return false.
///
/// Returns an error if the solang.toml file does not exist, or if the file cannot be read or parsed.
pub fn check_target_match(target_name: &str, config_file_content: Option<String>) -> Result<bool> {
    // Get the content of the configuration file
    // If the content is provided as an argument, then use it
    // Otherwise, read the content from the solang.toml file in the current directory
    let content = if let Some(content) = config_file_content {
        content
    } else {
        // Get the manifest path from the current directory
        let manifest_path = PathBuf::from("solang.toml");

        // Check if the manifest file exists
        // If it doesn't, then we don't need to check the target name
        if !manifest_path.exists() {
            return Ok(true);
        }

        // Read the content of the solang.toml file
        let mut file = File::open(&manifest_path).map_err(|err| {
            anyhow!(
                "Failed to open solang.toml file in the current directory: {}",
                err
            )
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|err| {
            anyhow!(
                "Failed to read solang.toml file in the current directory: {}",
                err
            )
        })?;

        content
    };

    // Parse the TOML content and extract the target name
    let parsed_toml: toml::Value = toml::from_str(&content).map_err(|err| {
        anyhow::anyhow!(
            "Failed to parse solang.toml file in the current directory: {}",
            err
        )
    })?;
    let config_target = parsed_toml["target"]["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to get target name from solang.toml"))?
        .to_string();

    // Compare the target name with the provided argument
    if config_target != target_name {
        eprintln!(
            "Error: The specified target '{}' does not match the target '{}' in solang.toml",
            target_name, config_target
        );
        return Ok(false);
    }

    Ok(true)
}

/// A test for the `check_target_match` function
#[test]
fn test_check_target_match() {
    // Test that the function returns true if the target names match
    let target_name = "solana";
    // Load the content of the solang.toml file from the Solana examples directory
    let config_file_content =
        include_str!("../solang_config_examples/solana_config.toml").to_string();
    let result = check_target_match(target_name, Some(config_file_content));
    assert!(result.is_ok(), "Error: {:?}", result);
    assert!(result.unwrap());

    // Test that the function returns false if the target names do not match
    let target_name = "solana";
    // Load the content of the solang.toml file from the Polkadot examples directory
    let config_file_content =
        include_str!("../solang_config_examples/polkadot_config.toml").to_string();
    let result = check_target_match(target_name, Some(config_file_content));
    assert!(result.is_ok(), "Error: {:?}", result);
    assert!(!result.unwrap());
}
