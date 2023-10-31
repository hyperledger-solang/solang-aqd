// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::Result,
    aqd_solana_contracts::{construct_instruction_data, idl_from_json},
    std::ffi::OsStr,
};

/// Purpose: This test checks the instruction data for the "new" function of the address program.
///
/// Note: The "address" program is a custom program that was created for this test.
/// The program's IDL is defined in tests/contracts/AddressTest.json.
/// The purpose of this program is to test address type.
#[tokio::test]
pub async fn test_address_new_data() -> Result<()> {
    // Define the address program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/AddressTest.json";
    let instruction_name = "new";
    let data = vec!["1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM".to_string()];

    // Load the address program's IDL and find the instruction we want to test.
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
        vec![
            135, 44, 205, 198, 25, 1, 72, 188, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    );

    Ok(())
}
