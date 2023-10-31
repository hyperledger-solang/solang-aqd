// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::Result,
    aqd_solana_contracts::{
        construct_instruction_accounts, construct_instruction_data, idl_from_json,
    },
    solana_sdk::pubkey::Pubkey,
    std::ffi::OsStr,
};

/// Purpose: This test checks the instruction data for the "new" function of the flipper program.
#[tokio::test]
pub async fn test_flipper_new_data() -> Result<()> {
    // Define the flipper program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/flipper.json";
    let instruction_name = "new";
    let data = vec!["true".to_string()];

    // Load the flipper program's IDL and find the instruction we want to test.
    let idl = idl_from_json(OsStr::new(idl_json))?;
    let idl_instruction =
        if let Some(instruction) = idl.instructions.iter().find(|i| i.name == instruction_name) {
            instruction.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Instruction not found: {}",
                instruction_name
            ));
        };
    let custom_types = idl.types.clone();

    // Construct the instruction data.
    let data = construct_instruction_data(&idl_instruction, &data, &custom_types)?;

    // Verify the instruction data is correct.
    assert_eq!(data, vec![135, 44, 205, 198, 25, 1, 72, 188, 1]);

    Ok(())
}

/// Purpose: This test checks the instruction data for the "get" function of the flipper program.
#[tokio::test]
pub async fn test_flipper_get_data() -> Result<()> {
    // Define the flipper program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/flipper.json";
    let instruction_name = "get";
    let data = vec![];

    // Load the flipper program's IDL and find the instruction we want to test.
    let idl = idl_from_json(OsStr::new(idl_json))?;
    let idl_instruction =
        if let Some(instruction) = idl.instructions.iter().find(|i| i.name == instruction_name) {
            instruction.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Instruction not found: {}",
                instruction_name
            ));
        };
    let custom_types = idl.types.clone();

    // Construct the instruction data.
    let data = construct_instruction_data(&idl_instruction, &data, &custom_types)?;

    // Verify the instruction data is correct.
    assert_eq!(data, vec![161, 224, 50, 61, 5, 210, 122, 216]);

    Ok(())
}

/// Purpose: This test checks the instruction accounts for the "get" function of the flipper program.
#[tokio::test]
pub async fn test_flipper_get_accounts() -> Result<()> {
    // Define the flipper program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/flipper.json";
    let instruction_name = "get";

    // Define the accounts.
    // For "get", the only account is the data account.
    let new_account = Pubkey::new_unique().to_string();
    let accounts = vec![new_account];

    // Load the flipper program's IDL and find the instruction we want to test.
    let idl = idl_from_json(OsStr::new(idl_json))?;
    let idl_instruction =
        if let Some(instruction) = idl.instructions.iter().find(|i| i.name == instruction_name) {
            instruction.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Instruction not found: {}",
                instruction_name
            ));
        };

    // Construct the instruction accounts.
    let (accounts, signers, new_accounts) =
        construct_instruction_accounts(&idl_instruction, &accounts)?;

    // Verify the instruction accounts are correct.

    // Only one account is returned, the data account.
    assert_eq!(accounts.len(), 1);
    // No signers are returned.
    assert_eq!(signers.len(), 0);
    // No new accounts are created.
    assert_eq!(new_accounts.len(), 0);
    // The data account is not signable nor mutable.
    assert_eq!(accounts[0].is_signer, false);
    assert_eq!(accounts[0].is_writable, false);

    Ok(())
}

/// Purpose: This test checks the instruction accounts for the "flip" function of the flipper program.
#[tokio::test]
pub async fn test_flipper_flip_accounts() -> Result<()> {
    // Define the flipper program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/flipper.json";
    let instruction_name = "flip";

    // Define the accounts.
    // For "flip", the only account is the data account.
    let new_account = Pubkey::new_unique().to_string();
    let accounts = vec![new_account];

    // Load the flipper program's IDL and find the instruction we want to test.
    let idl = idl_from_json(OsStr::new(idl_json))?;
    let idl_instruction =
        if let Some(instruction) = idl.instructions.iter().find(|i| i.name == instruction_name) {
            instruction.clone()
        } else {
            return Err(anyhow::anyhow!(
                "Instruction not found: {}",
                instruction_name
            ));
        };

    // Construct the instruction accounts.
    let (accounts, signers, new_accounts) =
        construct_instruction_accounts(&idl_instruction, &accounts)?;

    // Verify the instruction accounts are correct.

    // Only one account is returned, the data account.
    assert_eq!(accounts.len(), 1);
    // No signers are returned.
    assert_eq!(signers.len(), 0);
    // No new accounts are created.
    assert_eq!(new_accounts.len(), 0);
    // The data account is not signable but mutable.
    assert_eq!(accounts[0].is_signer, false);
    assert_eq!(accounts[0].is_writable, true);

    Ok(())
}
