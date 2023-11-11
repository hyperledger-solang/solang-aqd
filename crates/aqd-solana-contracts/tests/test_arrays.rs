// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::Result,
    aqd_solana_contracts::{construct_instruction_data, idl_from_json},
    std::ffi::OsStr,
};

/// Purpose: This test checks the instruction data for the "new" function of the "array" program.
///
/// Note: The "array" program is a custom program that was created for this test.
/// The program's IDL is defined in tests/contracts/array.json.
/// The purpose of this program is to test fixed array type.
#[tokio::test]
pub async fn test_array_data() -> Result<()> {
    // Define the program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/array.json";
    let instruction_name = "new";
    let data: Vec<String> = vec!["1,2,3,4".to_string()];

    // Load the program's IDL and find the instruction we want to test.
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
    assert_eq!(data, vec![135, 44, 205, 198, 25, 1, 72, 188, 1, 2, 3, 4]);

    Ok(())
}

/// Purpose: This test checks the instruction data for the "new" function of the "vector" program.
///
/// Note: The "VectorTest" program is a custom program that was created for this test.
/// The program's IDL is defined in tests/contracts/VectorTest.json.
/// The purpose of this program is to test fixed array type.
#[tokio::test]
pub async fn test_vector_data() -> Result<()> {
    // Define the program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/VectorTest.json";
    let instruction_name = "new";
    let data: Vec<String> = vec!["1,2,3".to_string()];

    // Load the program's IDL and find the instruction we want to test.
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
        vec![135, 44, 205, 198, 25, 1, 72, 188, 3, 0, 0, 0, 1, 2, 3]
    );

    Ok(())
}

/// Purpose: This test checks the instruction data for the "new" function of the "byte_test" program.
///
/// Note: The "byte_test" program is a custom program that was created for this test.
/// The program's IDL is defined in tests/contracts/byte_test.json.
/// The purpose of this program is to test bytes type.
#[tokio::test]
pub async fn test_bytes_data() -> Result<()> {
    // Define the program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/byte_test.json";
    let instruction_name = "new";
    let data: Vec<String> = vec!["123456".to_string()];

    // Load the program's IDL and find the instruction we want to test.
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
        vec![135, 44, 205, 198, 25, 1, 72, 188, 3, 0, 0, 0, 18, 52, 86]
    );

    Ok(())
}
