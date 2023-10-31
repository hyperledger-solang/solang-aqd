// SPDX-License-Identifier: Apache-2.0

use {
    anyhow::Result,
    aqd_solana_contracts::{construct_instruction_data, idl_from_json},
    std::ffi::OsStr,
};

/// Purpose: This test checks the instruction data for the "new" function of the "unsigned_int" program.
///
/// Note: The "unsigned_int" program is a custom program that was created for this test.
/// The program's IDL is defined in tests/contracts/unsigned_int.json.
/// The purpose of this program is to test unsigned integer types.
#[tokio::test]
pub async fn test_unsigned_int_data() -> Result<()> {
    // Define the program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/unsigned_int.json";
    let instruction_name = "new";
    let data: Vec<String> = vec![
        // u8
        "12".to_string(),
        // u16
        "1234".to_string(),
        // u32
        "12345678".to_string(),
        // u64
        "123456789012345".to_string(),
        // u128
        "123456789012345678901234".to_string(),
        // u256
        "1234567890123456789012345678901234567890".to_string(),
    ];

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
        vec![
            135, 44, 205, 198, 25, 1, 72, 188, 12, 210, 4, 78, 97, 188, 0, 121, 223, 13, 134, 72,
            112, 0, 0, 242, 175, 150, 108, 160, 16, 31, 155, 36, 26, 0, 0, 0, 0, 0, 0, 210, 10, 63,
            206, 150, 95, 188, 172, 184, 243, 219, 192, 117, 32, 201, 160, 3, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0
        ]
    );

    Ok(())
}

/// Purpose: This test checks the instruction data for the "new" function of the "signed_int" program.
///
/// Note: The "signed_int" program is a custom program that was created for this test.
/// The program's IDL is defined in tests/contracts/signed_int.json.
/// The purpose of this program is to test signed integer types.
#[tokio::test]
pub async fn test_signed_int_data() -> Result<()> {
    // Define the program's IDL and the instruction we want to test.
    let idl_json = "tests/contracts/signed_int.json";
    let instruction_name = "new";
    let data: Vec<String> = vec![
        // i8
        "12".to_string(),
        // i16
        "-1234".to_string(),
        // i32
        "12345678".to_string(),
        // i64
        "-123456789012345".to_string(),
        // i128
        "123456789012345678901234".to_string(),
        // i256
        "-1234567890123456789012345678901234567890".to_string(),
    ];

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
        vec![
            135, 44, 205, 198, 25, 1, 72, 188, 12, 46, 251, 78, 97, 188, 0, 135, 32, 242, 121, 183,
            143, 255, 255, 242, 175, 150, 108, 160, 16, 31, 155, 36, 26, 0, 0, 0, 0, 0, 0, 46, 245,
            192, 49, 105, 160, 67, 83, 71, 12, 36, 63, 138, 223, 54, 95, 252, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255
        ]
    );

    Ok(())
}
