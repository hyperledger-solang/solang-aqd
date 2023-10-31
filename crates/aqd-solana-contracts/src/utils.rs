// SPDX-License-Identifier: Apache-2.0

use {
    crate::borsh_encoding::{discriminator, encode_arguments, BorshToken},
    anchor_syn::idl::{
        Idl, IdlAccountItem, IdlInstruction, IdlType, IdlTypeDefinition, IdlTypeDefinitionTy::Enum,
        IdlTypeDefinitionTy::Struct,
    },
    anyhow::{anyhow, bail, Result},
    base58::FromBase58,
    num_bigint::BigInt,
    solana_cli_config::{Config, CONFIG_FILE},
    solana_sdk::{
        instruction::AccountMeta,
        pubkey::Pubkey,
        signature::{write_keypair_file, Keypair, Signer},
        signer::keypair::read_keypair_file,
        system_program,
    },
    std::{ffi::OsStr, fs::File, str::FromStr},
};

/// Parses an IDL (Interface Description Language) definition from a JSON file.
///
/// Given a file path provided as an [`OsStr`], this function attempts to open the file and
/// parse its content into an [`Idl`] struct. If successful,
/// it returns the parsed [`Idl`] struct.
///
/// # Arguments
///
/// * `file` - The path to the JSON file containing the IDL instruction definition.
///
/// # Errors
///
/// This function can return errors if the provided file path is invalid, the file cannot be
/// opened, or there are issues with parsing the JSON content.
pub fn idl_from_json(file: &OsStr) -> Result<Idl> {
    let f = match File::open(file) {
        Ok(s) => s,
        Err(e) => {
            bail!("{}: error: {}", file.to_string_lossy(), e);
        }
    };

    // Parse the JSON into an IDL
    match serde_json::from_reader(f) {
        Ok(idl) => Ok(idl),
        Err(e) => {
            bail!("{}: error: {}", file.to_string_lossy(), e);
        }
    }
}

/// Constructs accounts, keypairs, and new accounts information for an IDL instruction.
///
/// Given an [`IdlInstruction`] and a vector of raw account arguments, this function processes
/// the arguments, creates the necessary [`AccountMeta`] instances, and determines which accounts
/// should be treated as signers. If the raw account argument is one of the following keywords,
/// special actions are taken:
///
/// - `new`: Create a new account and generate a keypair for it. The account's public key and
///   keypair path are recorded for reference.
///
/// - `self`: Use the keypair specified in the local solana configuration file.
///
/// - `system`: Use the system program ID for the account. This is equivalent to passing in the
///  system program ID as a public key.
///
/// For other raw account arguments, the function checks if it's a valid keypair path or a valid
/// public key. If it's a valid keypair path, the keypair is loaded and used for the account. If
/// it's a valid public key, the public key is used for the account. Otherwise, an error is
/// returned.
///
/// # Arguments
///
/// * `instr` - The IDL instruction of type [`IdlInstruction`] for which to construct accounts.
///
/// * `raw_args` - A vector of raw account arguments. Each argument can be one of the keywords
///   mentioned above, a keypair path, or a public key.
///
/// # Returns
///
/// Returns a `Result` containing a tuple of three vectors:
///
/// 1. A vector of [`AccountMeta`] instances representing the accounts required for the instruction.
///
/// 2. A vector of [`Keypair`] instances representing signers among the accounts.
///
/// 3. A vector of `(Pubkey, String)` pairs representing new accounts created during the process.
///
/// # Errors
///
/// This function can return an error in the following cases:
///
/// - If a raw account argument is missing or invalid.
///
/// - If an account type is a nested account (e.g., `IdlAccounts`).
///
/// - If the provided argument for an account is not a valid keyword, keypair path, or public key.
#[allow(clippy::type_complexity)]
pub fn construct_instruction_accounts(
    instr: &IdlInstruction,
    raw_args: &[String],
) -> Result<(Vec<AccountMeta>, Vec<Keypair>, Vec<(Pubkey, String)>)> {
    // Initialize the return values
    let mut accounts: Vec<AccountMeta> = vec![];
    let mut signers: Vec<Keypair> = vec![];
    let mut new_accounts: Vec<(Pubkey, String)> = vec![];

    // Loop through the accounts and create the account meta
    // given the raw arguments
    for (i, account) in instr.accounts.iter().enumerate() {
        let (account_name, is_signer, is_writable) = match account {
            IdlAccountItem::IdlAccount(account) => {
                (account.name.clone(), account.is_signer, account.is_mut)
            }
            IdlAccountItem::IdlAccounts(_) => return Err(anyhow!("Nested accounts not supported")),
        };
        let raw_pubkey = raw_args
            .get(i)
            .ok_or_else(|| anyhow!("Missing account: {}", account_name))?;
        let (key_pair, pubkey) = match raw_pubkey.as_str() {
            "new" => {
                // "new" is a special keyword that creates a new account
                let keypair = Keypair::new();
                let pubkey = keypair.pubkey();
                // Write the keypair to a file
                let keypair_path = format!("{}-{}.json", account_name, pubkey);
                write_keypair_file(&keypair, &keypair_path)
                    .map_err(|_| anyhow!("Couldn't write keypair file to disk"))?;
                new_accounts.push((pubkey, keypair_path.clone()));

                (Some(keypair), pubkey)
            }
            "self" => {
                // "self" is a special keyword that uses the keypair from the config file
                let config_file = CONFIG_FILE.as_ref().unwrap();
                let cli_config = Config::load(config_file).unwrap_or_default();
                let keypair = read_keypair_file(&cli_config.keypair_path).unwrap();
                let pubkey = keypair.pubkey();
                (Some(keypair), pubkey)
            }
            "system" => (
                // "system" is a special keyword that uses the system program ID
                None,
                system_program::id(),
            ),
            // There are 2 cases here:
            // 1. The user passes in a keypair path
            // 2. The user passes in a public key
            _ => {
                // First, check if the user passed in a keypair path
                let keypair = read_keypair_file(raw_pubkey);
                match keypair {
                    Ok(keypair) => {
                        let pubkey = keypair.pubkey();
                        (Some(keypair), pubkey)
                    }
                    Err(_) => {
                        // The user passed in a public key
                        let pubkey = Pubkey::from_str(raw_pubkey).map_err(|_e| {
                            anyhow!("The provided argument for account: {} is not a valid keyword, keypair path or public key. \nProvided argument: {}", account_name , raw_pubkey)
                        })?;
                        (None, pubkey)
                    }
                }
            }
        };

        // Add the keypair to the list of signers if the account is a signer
        if is_signer {
            let key_pair = key_pair.ok_or_else(
                || anyhow!("The provided argument for account: {} is not a valid keyword, keypair path or public key. \nProvided argument: {}", account_name , raw_pubkey),
            )?; // This should never fail
            signers.push(key_pair);
        }
        accounts.push(AccountMeta {
            pubkey,
            is_signer,
            is_writable,
        });
    }

    Ok((accounts, signers, new_accounts))
}

/// Constructs binary data for an instruction based on the provided IDL instruction and raw arguments.
///
/// Given an [`IdlInstruction`], a vector of raw arguments, and a list of IDL type definitions, this
/// function encodes the arguments into binary data following the instruction's layout and type
/// definitions. The resulting binary data can be used as an instruction to invoke a smart contract
/// on a blockchain.
///
/// # Arguments
///
/// * `instr` - The IDL instruction of type [`IdlInstruction`] for which to construct binary data.
///
/// * `raw_args` - A vector of raw argument values represented as strings.
///
/// * `custom_types` - A vector of IDL type definitions used for encoding arguments.
///
/// # Returns
///
/// Returns a [`Result`] containing the encoded binary data as a [`Vec<u8>`].
///
/// # Errors
///
/// This function can return an error in the following cases:
///
/// - If any argument is missing or if the provided raw arguments do not match the expected
///   arguments defined by the IDL instruction.
///
/// - If the IDL-defined types cannot be found or if there is an issue with encoding the arguments
///   based on these types.
pub fn construct_instruction_data(
    instr: &IdlInstruction,
    raw_args: &[String],
    custom_types: &Vec<IdlTypeDefinition>,
) -> Result<Vec<u8>> {
    // Construct the discriminator (the first 8 bytes of the instruction data)
    // The namespace is always "global"
    let mut data = discriminator("global", &instr.name);
    let mut args: Vec<BorshToken> = vec![];

    // Loop through the arguments and encode them given the IDL instruction
    for (i, arg) in instr.args.iter().enumerate() {
        let arg_name = arg.name.clone();
        let arg_type = arg.ty.clone();
        let arg_val = raw_args
            .get(i)
            .ok_or_else(|| anyhow!("Missing argument {}", arg_name))?;

        // Encode the argument based on the IDL type
        let mut borsh_args = get_borsh_token_vector(arg_val.to_string(), &arg_type, custom_types)?;
        args.append(&mut borsh_args);
    }

    let mut encoded_data = encode_arguments(&args);
    data.append(&mut encoded_data);

    Ok(data)
}

/// Converts a raw argument value into a vector of Borsh tokens based on the provided IDL type.
///
/// This function takes a raw argument value as a string, an IDL type definition, and a list of
/// IDL type definitions for reference. It then converts the raw argument value into a vector of
/// Borsh tokens, considering the specified IDL type and nested types if applicable.
///
/// # Arguments
///
/// * `arg_value` - The raw argument value to be converted. The expected input format depends on
/// the IDL type:
///
///   - For structs, provide a JSON string representing the data structure.
///   - For arrays and vectors, provide a comma-separated string of values.
///
/// * `arg_type` - The IDL type definition specifying the expected type of the argument.
///
/// * `custom_types` - A vector of IDL type definitions used for resolving nested types.
///
/// # Returns
///
/// Returns a [`Result`] containing the vector of Borsh tokens ([`Vec<BorshToken>`]) representing
/// the converted argument value.
///
/// # Errors
///
/// This function can return an error if there is an issue with converting the raw argument value
/// to the specified Borsh tokens based on the given IDL type.
fn get_borsh_token_vector(
    arg_value: String,
    arg_type: &IdlType,
    custom_types: &Vec<IdlTypeDefinition>,
) -> Result<Vec<BorshToken>> {
    let mut args: Vec<BorshToken> = vec![];
    match arg_type {
        IdlType::Bool => {
            let val = arg_value.parse::<bool>().map_err(|_e| {
                anyhow!("The provided argument for bool is not a valid boolean. \nProvided argument: {}\n", arg_value)
            })?;
            args.push(BorshToken::Bool(val));
        }
        IdlType::U8 => {
            let val = arg_value.parse::<u8>().map_err(|_e| {
                anyhow!("The provided argument for u8 is not a valid unsigned integer. \nProvided argument: {}\n", arg_value)
            })?;
            args.push(BorshToken::Uint {
                width: 8,
                value: BigInt::from(val),
            });
        }
        IdlType::I8 => {
            let val = arg_value.parse::<i8>().map_err(|_e| {
                anyhow!("The provided argument for i8 is not a valid signed integer. \nProvided argument: {}\n", arg_value)
            })?;
            args.push(BorshToken::Int {
                width: 8,
                value: BigInt::from(val),
            });
        }
        IdlType::U16 => {
            let val = arg_value.parse::<u16>().map_err(|_e| {
                anyhow!("The provided argument for u16 is not a valid unsigned integer. \nProvided argument: {}\n", arg_value)
            })?;
            args.push(BorshToken::Uint {
                width: 16,
                value: BigInt::from(val),
            });
        }
        IdlType::I16 => {
            let val = arg_value.parse::<i16>().map_err(|_e| {
                anyhow!("The provided argument for i16 is not a valid signed integer. \nProvided argument: {}\n", arg_value)
            })?;
            args.push(BorshToken::Int {
                width: 16,
                value: BigInt::from(val),
            });
        }
        IdlType::U32 => {
            let val = arg_value.parse::<u32>().map_err(
                |_e| anyhow!("The provided argument for u32 is not a valid unsigned integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Uint {
                width: 32,
                value: BigInt::from(val),
            });
        }
        IdlType::I32 => {
            let val = arg_value.parse::<i32>().map_err(
                |_e| anyhow!("The provided argument for i32 is not a valid signed integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Int {
                width: 32,
                value: BigInt::from(val),
            });
        }
        IdlType::F32 => {
            return Err(anyhow!("Float is not supported"));
        }
        IdlType::U64 => {
            let val = arg_value.parse::<u64>().map_err(
                |_e| anyhow!("The provided argument for u64 is not a valid unsigned integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Uint {
                width: 64,
                value: BigInt::from(val),
            });
        }
        IdlType::I64 => {
            let val = arg_value.parse::<i64>().map_err(
                |_e| anyhow!("The provided argument for i64 is not a valid signed integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Int {
                width: 64,
                value: BigInt::from(val),
            });
        }
        IdlType::F64 => {
            return Err(anyhow!("Float is not supported"));
        }
        IdlType::U128 => {
            let val = arg_value.parse::<u128>().map_err(
                |_e| anyhow!("The provided argument for u128 is not a valid unsigned integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Uint {
                width: 128,
                value: BigInt::from(val),
            });
        }
        IdlType::I128 => {
            let val = arg_value.parse::<i128>().map_err(
                |_e| anyhow!("The provided argument for i128 is not a valid signed integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Int {
                width: 128,
                value: BigInt::from(val),
            });
        }
        IdlType::U256 => {
            let val = BigInt::parse_bytes(arg_value.as_bytes(), 10).ok_or_else(
                || anyhow!("The provided argument for u256 is not a valid unsigned integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Uint {
                width: 256,
                value: val,
            });
        }
        IdlType::I256 => {
            let val = BigInt::parse_bytes(arg_value.as_bytes(), 10).ok_or_else(
                || anyhow!("The provided argument for i256 is not a valid unsigned integer. \nProvided argument: {}\n", arg_value),
            )?;
            args.push(BorshToken::Int {
                width: 256,
                value: val,
            });
        }
        IdlType::Bytes => {
            let val = match hex::decode(&arg_value) {
                Ok(val) => val,
                Err(_) => {
                    return Err(anyhow!("The provided argument for Bytes is not a valid hex string. \nProvided argument: {}\n", arg_value))
                }
            };
            args.push(BorshToken::Bytes(val));
        }
        IdlType::String => {
            args.push(BorshToken::String(arg_value.to_string()));
        }
        IdlType::PublicKey => {
            let val = arg_value.from_base58().map_err(|_e| {
                anyhow!("The provided argument for PublicKey is not a valid base58 string. \nProvided argument: {}\n", arg_value)
            })?;
            let val = <[u8; 32]>::try_from(val).map_err(|_e| {
                anyhow!("The provided argument for PublicKey is not a valid base58 string. \nProvided argument: {}\n", arg_value)
            })?;
            args.push(BorshToken::Address(val));
        }
        IdlType::Defined(ty) => {
            let defined_type = custom_types
                .iter()
                .find(|t| t.name == *ty)
                .ok_or_else(|| anyhow!("Type definition with name {} not found", ty))?;
            let mut borsh_args_for_defined_type =
                encode_id_defined_type(arg_value.to_string(), defined_type, custom_types)?;
            args.append(&mut borsh_args_for_defined_type);
        }
        IdlType::Option(_) => {
            return Err(anyhow!("Option is not supported"));
        }
        IdlType::Vec(elem_type) => {
            // Split the string into a vector of strings
            let val: Vec<String> = arg_value.split(',').map(|s| s.to_string()).collect();
            let mut borsh_args: Vec<BorshToken> = vec![];
            for arg in val {
                let mut borsh_arg = get_borsh_token_vector(arg, elem_type, custom_types)?;
                borsh_args.append(&mut borsh_arg);
            }
            args.push(BorshToken::Array(borsh_args));
        }
        IdlType::Array(elem_type, size) => {
            // Split the string into a vector of strings
            let val: Vec<String> = arg_value.split(',').map(|s| s.to_string()).collect();
            // Make sure the number of elements matches the size of the array
            if val.len() != *size {
                return Err(anyhow!(
                    "The number of elements in the array does not match the size of the array. \nProvided argument: {}\n",
                    arg_value
                ));
            }
            let mut borsh_args: Vec<BorshToken> = vec![];
            for arg in val {
                let mut borsh_arg = get_borsh_token_vector(arg, elem_type, custom_types)?;
                borsh_args.append(&mut borsh_arg);
            }
            args.push(BorshToken::FixedArray(borsh_args));
        }
    }
    Ok(args)
}

/// Converts a raw argument value into a vector of Borsh tokens for a custom IDL-defined type.
///
/// This function takes a raw argument value as a string, a custom IDL type definition, and a list
/// of IDL type definitions for reference. It then converts the raw argument value into a vector of
/// Borsh tokens based on the specified custom IDL type, including handling structs and enums as per
/// the IDL definition.
///
/// # Arguments
///
/// * `arg_value` - The raw argument value as a string to be converted.
///
/// * `defined_type` - The custom IDL type definition specifying the expected type of the argument.
///
/// * `custom_types` - A vector of IDL type definitions used for resolving nested types.
///
/// # Returns
///
/// Returns a [`Result`] containing the vector of Borsh tokens ([`Vec<BorshToken>`]) representing
/// the converted argument value.
fn encode_id_defined_type(
    arg_value: String,
    defined_type: &IdlTypeDefinition,
    custom_types: &Vec<IdlTypeDefinition>,
) -> Result<Vec<BorshToken>> {
    let mut response: Vec<BorshToken> = vec![];
    let ty = &defined_type.ty;
    match ty {
        Struct { fields } => {
            // The user could either pass a JSON object or a path to a JSON file
            // If the user passes a path to a JSON file, we need to read the file first
            let arg_value = match std::fs::read_to_string(&arg_value) {
                Ok(s) => s,
                Err(_) => arg_value,
            };

            // The user should pass a json object
            // for example: {"a": 1, "b": 2}
            // we need to parse the json object and then encode it
            let json_object: serde_json::Value = serde_json::from_str(&arg_value).map_err(
                |_e| anyhow!("The provided argument for Struct is not a valid JSON object. \nProvided argument: {}\n", arg_value),
            )?;
            for field in fields {
                let field_name = &field.name;
                let field_ty = &field.ty;
                let field_value = json_object
                    .get(field_name)
                    .ok_or_else(|| anyhow!("Field {} not found", field_name))?;
                let mut borsh_args =
                    get_borsh_token_vector(field_value.to_string(), field_ty, custom_types)?;
                response.append(&mut borsh_args);
            }
        }
        Enum { variants } => {
            // If the variant is a string, we need to remove the quotes
            let arg_value = arg_value.replace('\"', "");
            // The user passes a String of the variant name
            // for example: "A"
            // we need to find index of the variant and then encode it
            let variant_index = variants
                .iter()
                .position(|v| v.name == arg_value)
                .ok_or_else(|| {
                    anyhow!(
                        "Variant {} not found. \nAvailable variants of {}: {:?}",
                        arg_value,
                        defined_type.name,
                        variants
                            .iter()
                            .map(|v| v.clone().name)
                            .collect::<Vec<String>>()
                    )
                })?;

            let mut borsh_args =
                get_borsh_token_vector(variant_index.to_string(), &IdlType::U8, custom_types)?;
            response.append(&mut borsh_args);
        }
    }

    Ok(response)
}
