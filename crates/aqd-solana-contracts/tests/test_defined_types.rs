// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::Result,
    aqd_solana_contracts::{construct_instruction_data, idl_from_json},
    std::ffi::OsStr,
};

/// Purpose: This test checks the instruction data for the "new" function of the defined_types program.

/// Note: The "DefinedTypes" program is a custom program that was created for this test.
/// The program's IDL is defined in tests/contracts/DefinedTypes.json.
/// The purpose of this program is to test struct and Enum types.
#[tokio::test]
pub async fn test_defined_types_new_data() -> Result<()> {
    // Define the defined_types program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/DefinedTypes.json";
    let instruction_name = "new";
    let person = r#"
    {
        "name": "Alice",
        "age": 30,
        "favoriteColor": "Red"
      }"#
    .to_string();
    let data = vec![person];

    // Load the defined_types program's IDL and find the instruction we want to test.
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
    assert_eq!(
        data,
        vec![135, 44, 205, 198, 25, 1, 72, 188, 7, 0, 0, 0, 34, 65, 108, 105, 99, 101, 34, 30, 0]
    );

    Ok(())
}
