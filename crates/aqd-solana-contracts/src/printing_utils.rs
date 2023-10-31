// SPDX-License-Identifier: Apache-2.0

// To be able to use (base64::decode) in `decode_transaction_return_data` function
#![allow(deprecated)]

use {
    crate::borsh_encoding::decode_at_offset,
    anchor_syn::idl::{Idl, IdlAccountItem, IdlInstruction, IdlTypeDefinition},
    anyhow::{anyhow, Result},
    aqd_utils::{print_key_value, print_subtitle, print_title, print_value},
    colored::Colorize,
    serde_json::{json, Map, Value},
    solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig},
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature,
        transaction::TransactionVersion::Legacy, transaction::TransactionVersion::Number,
    },
    solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding},
};

/// Prints information about instructions in an Instruction Description Language (IDL) definition.
///
/// This function takes an [`Idl`] structure, an optional instruction name, and a flag for output format.
/// It provides information about instructions defined in the [`Idl`]. If an instruction name is provided,
/// it will print details about that specific instruction. Otherwise, it can print information about all
/// instructions in the [`Idl`]. The information includes the instruction name, documentation,
/// associated accounts, and arguments. The output format can be either JSON or human-readable.
///
/// The function will print information about the instruction, its associated accounts, and arguments based on the
/// specified output format.
///
/// # Arguments
///
/// * `idl`: A reference to an [`Idl`] structure that defines the instructions.
/// * `instruction_name`: An optional reference to a specific instruction name to print details for.
/// * `output_json`: A boolean flag indicating whether to output the information in JSON format.
pub fn print_idl_instruction_info(idl: &Idl, instruction_name: Option<String>, output_json: bool) {
    // If the instruction name is provided, print only that instruction
    if let Some(instruction_name) = instruction_name {
        // Find the instruction with the specified name
        if let Some(instruction) = idl
            .instructions
            .iter()
            .find(|i| i.name == *instruction_name)
        {
            print_single_instruction_info(instruction, output_json);
        } else {
            eprintln!("Instruction {} not found", instruction_name);
        }
    } else {
        // Print all instructions' information
        if output_json {
            // This is to ensure that we print only 1 JSON
            let val = match serde_json::to_string_pretty(&idl.instructions) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return;
                }
            };
            println!("{}", val);
        } else {
            for instruction in idl.instructions.iter() {
                print_single_instruction_info(instruction, output_json);
            }
        }
    }
}

/// Print detailed information about an instruction.
///
/// This function takes an instruction and an output format flag. It prints comprehensive details
/// about the given instruction, including its name, documentation, associated accounts, and arguments.
/// The output format can be either JSON or human-readable.
fn print_single_instruction_info(instruction: &IdlInstruction, output_json: bool) {
    if output_json {
        match serde_json::to_string_pretty(&instruction) {
            Ok(val) => println!("{}", val),
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        };
    } else {
        // Print the instruction name
        print_title!("Instruction name");
        print_value!(instruction.name);

        // Print the instruction documentation
        print_title!("Instruction docs");
        let docs = match &instruction.docs {
            Some(docs) => docs.join("\n"),
            None => "No documentation".to_string(),
        };
        print_value!(docs);

        // Print the associated accounts
        print_title!("Accounts");
        // If there are no accounts, print a message
        if instruction.accounts.is_empty() {
            print_value!("No accounts");
        }
        // Loop through the accounts and print their details
        for (i, account) in instruction.accounts.iter().enumerate() {
            let key: String = format!("Account {}", i + 1);
            print_subtitle!(key);
            match account {
                IdlAccountItem::IdlAccount(account) => {
                    print_key_value!("Account Name: ", format!("{}", account.name));
                    print_key_value!("Is signer: ", format!("{}", account.is_signer));
                    print_key_value!("Is mutable: ", format!("{}", account.is_mut));
                    print_key_value!("Is optional: ", format!("{:?}", account.is_optional));
                    print_key_value!("Account docs: ", format!("{:?}", account.docs));
                    print_key_value!("Account PDA: ", format!("{:?}", account.pda));
                }
                IdlAccountItem::IdlAccounts(accounts) => {
                    // Print a warning that this is a nested account
                    println!("{}", "Nested accounts are not supported".red());
                    let accounts_str = format!("{:?}", accounts);
                    print_key_value!(key, accounts_str);
                }
            }
        }

        // Print the instruction arguments
        print_title!("Args");
        // If there are no arguments, print a message
        if instruction.args.is_empty() {
            print_value!("No arguments");
        }
        // Loop through the arguments and print their details
        for (i, arg) in instruction.args.iter().enumerate() {
            let key = format!("Arg {}", i + 1);
            print_subtitle!(key);
            print_key_value!("Arg name: ", format!("{}", arg.name));
            print_key_value!("Arg type: ", format!("{:?}", arg.ty));
            print_key_value!("Arg docs: ", format!("{:?}", arg.docs));
        }
    }
}

/// Print transaction information given a transaction signature.
///
/// The function prints detailed information about the transaction, including the instruction name, associated accounts,
/// and arguments. The output format can be either JSON or human-readable.
///
/// # Arguments
///
/// * `rpc_client`: A reference to the [`RpcClient`] used to communicate with the Solana cluster.
/// * `signature`: A reference to the transaction [`Signature`] to retrieve transaction details.
/// * `instruction`: A reference to the [`IdlInstruction`] representing the instruction in the transaction.
/// * `custom_types`: An array of custom [`IdlTypeDefinition`]s used in the IDL definition.
/// * `new_accounts`: A reference to a list of new accounts as tuples containing the [`Pubkey`] and keypair file path.
/// * `output_json`: A boolean flag indicating whether to output the information in JSON format.
///
/// The function will print information about the transaction, the associated instruction, its accounts, and arguments
/// based on the specified output format.
pub fn print_transaction_information(
    rpc_client: &RpcClient,
    signature: &Signature,
    instruction: &IdlInstruction,
    custom_types: &[IdlTypeDefinition],
    new_accounts: &Vec<(Pubkey, String)>,
    output_json: bool,
) -> Result<()> {
    // If the instruction has a return value, we need to decode it using the IDL definition
    let decoded_return_data =
        decode_instruction_return_data(rpc_client, signature, instruction, custom_types)?
            .unwrap_or("None".to_string());

    if output_json {
        // For the JSON output, we need to fetch the transaction details using the RpcTransactionConfig
        // with the encoding set to JSON or JSONParsed
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };
        let transaction = rpc_client.get_transaction_with_config(signature, config)?;
        let transaction_info = transaction.transaction;

        // Deserialize the transaction to a JSON object
        let mut transaction_json: Map<String, Value> =
            serde_json::from_str(&serde_json::to_string(&transaction_info)?)?;

        // If new accounts were created, add them to the JSON transaction
        // instead of printing them separately.
        // This is to ensure that we print only 1 JSON.
        if !new_accounts.is_empty() {
            // Create a JSON array of new accounts
            let new_accounts_json: Vec<Value> = new_accounts
                .iter()
                .map(|(pubkey, name)| {
                    json!({
                        "pubkey": pubkey.to_string(),
                        "file_name": name,
                    })
                })
                .collect();

            // Add new_accounts field to the JSON transaction
            transaction_json.insert("new_accounts".to_string(), Value::Array(new_accounts_json));
        }

        transaction_json.insert(
            "decoded_return_data".to_string(),
            Value::String(decoded_return_data),
        );

        // Serialize the modified transaction back to a string
        let modified_pretty_trans = serde_json::to_string_pretty(&Value::Object(transaction_json))?;
        println!("{}", modified_pretty_trans);
    } else {
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: None,
        };
        let transaction = rpc_client.get_transaction_with_config(signature, config)?;
        let transaction_info = transaction.transaction;

        if let Some(trans) = transaction_info.transaction.decode() {
            // Print the transaction version
            let version = trans.version();
            let version = match version {
                Legacy(_) => "Legacy".to_string(),
                Number(val) => format!("Number: {}", val),
            };
            print_title!("Transaction version");
            print_value!(version);

            // Print the signatures
            let signatures = trans.signatures;
            print_title!("Signatures");
            for (i, signature) in signatures.iter().enumerate() {
                let key = format!("Signature {}", i + 1);
                print_key_value!(key, signature);
            }

            // Print the message
            let message = trans.message;

            // Print the message header
            let message_header = message.header();
            print_title!("Message header");
            print_key_value!(
                "Number of required signatures",
                message_header.num_required_signatures
            );
            print_key_value!(
                "Number of readonly signed accounts",
                message_header.num_readonly_signed_accounts
            );
            print_key_value!(
                "Number of readonly unsigned accounts",
                message_header.num_readonly_unsigned_accounts
            );

            // Print the message account keys
            let account_keys = message.static_account_keys();
            print_title!("Account keys");
            for (i, account_key) in account_keys.iter().enumerate() {
                let key = format!("Account key {}", i + 1);
                print_key_value!(key, account_key);
            }

            // Print the message recent block hash
            let recent_block_hash = message.recent_blockhash();
            print_title!("Recent block hash");
            print_value!(recent_block_hash);

            // Print the message instructions
            let instructions = message.instructions();
            print_title!("Instructions");
            for (i, instruction) in instructions.iter().enumerate() {
                let program_id_index = instruction.program_id_index;
                let accounts = &instruction.accounts;
                let data = &instruction.data;
                print_subtitle!(format!("Instruction {}", i + 1));
                print_key_value!("Program ID index", program_id_index);
                let accounts = format!("{:?}", accounts);
                print_key_value!("Accounts", accounts);
                let data = format!("{:?}", data);
                print_key_value!("Data", data);
            }
            // Print the new accounts (if any)
            if !new_accounts.is_empty() {
                print_title!("New accounts");
                for (i, (pubkey, name)) in new_accounts.iter().enumerate() {
                    print_subtitle!(format!("New account {}", i + 1));
                    print_key_value!("Pubkey", pubkey);
                    print_key_value!("File name", name);
                }
            }
        } else {
            return Err(anyhow!("Error decoding transaction"));
        }

        // Print transaction return data
        if let Some(transaction_status) = transaction_info.meta {
            // Print the transaction status
            let status = match transaction_status.status {
                Ok(_) => "Ok".to_string(),
                Err(_) => "Error".to_string(),
            };
            let err = transaction_status.err;
            print_title!("Transaction status");
            print_key_value!("Status", status);
            if let Some(err) = err {
                print_key_value!("Error", err);
            }

            // Print the transaction return data
            print_title!("Transaction return data");
            print_value!(decoded_return_data);

            // Print the transaction logs
            let logs = transaction_status.log_messages;
            match logs {
                OptionSerializer::Some(val) => {
                    print_subtitle!("Logs");
                    for log in val {
                        print_value!(log);
                    }
                }
                OptionSerializer::None | OptionSerializer::Skip => {}
            }
        }
    }
    Ok(())
}

/// Decode the return data from a Solana instruction.
///
/// Given the `rpc_client`, `signature` of the transaction containing the instruction, the
/// `instruction` description, and a vector of custom `custom_types`, this function attempts to
/// decode the return data of the instruction. If the instruction has no return value, it returns
/// `None`.
///
/// The return data is extracted from the transaction logs, where it's identified as "Program
/// return." The encoded data is Base64 encoded, and this function decodes it and attempts to
/// deserialize it according to the provided `instruction` and `custom_types`.
///
/// If successful, it returns the decoded data as a string wrapped in an `Ok` variant. If any error
/// occurs during the decoding process, it returns an `Err` variant containing an error message.
///
/// # Parameters
///
/// - `rpc_client`: A reference to the Solana RPC client of type [`RpcClient`].
/// - `signature`: The transaction signature containing the instruction of type [`Signature`].
/// - `instruction`: A reference to the instruction description of type [`IdlInstruction`].
/// - `custom_types`: A vector of custom IDL type definitions used for deserialization of type [`IdlTypeDefinition`].
///
/// # Returns
///
/// - `Ok(Some(result))`: The decoded return data as a string if successful.
/// - `Ok(None)`: If the instruction has no return value.
/// - `Err(error)`: If an error occurs during the decoding process.
pub fn decode_instruction_return_data(
    rpc_client: &RpcClient,
    signature: &Signature,
    instruction: &IdlInstruction,
    custom_types: &[IdlTypeDefinition],
) -> Result<Option<String>> {
    // If the instruction has no return value, return None
    let ty = instruction.returns.as_ref();
    if ty.is_none() {
        return Ok(None);
    }
    // This can be unwrapped safely because we checked that it's not None
    let ty = ty.unwrap();

    // Fetch the transaction details using the RpcTransactionConfig
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Base64),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: None,
    };
    let transaction = rpc_client.get_transaction_with_config(signature, config)?;
    let transaction_meta = transaction
        .transaction
        .meta
        .ok_or_else(|| anyhow!("Error fetching transaction return data from transaction meta"))?;

    // We need to extract the return data from the logs.
    // can't use the return data from the `EncodedTransactionWithStatusMeta` directly
    // because of an issue fixed in this PR:
    // https://github.com/solana-labs/solana/pull/33639
    // This is a workaround until a new version of the solana crate is released
    // with the fix.

    let logs = transaction_meta.log_messages;

    let mut response = String::new();
    match logs {
        OptionSerializer::Some(val) => {
            for log in val {
                if log.contains("Program return") {
                    // A sample log message containing return data:
                    // "Program return: FiyfwwVZjuC2GE15X68fpKdA9SukqB7bk472FageXVGv AQ=="
                    // We need to extract the base64 encoded data
                    let data = log.split_whitespace().last().ok_or_else(|| {
                        anyhow!("Error extracting transaction return data from log")
                    })?;
                    // Deserialize the data from base64
                    let data = base64::decode(data)
                        .map_err(|e| anyhow!("Error decoding transaction return data: {}", e))?;
                    let data = data.as_slice();
                    let mut offset = 0;
                    let data = decode_at_offset(data, &mut offset, ty, custom_types).to_string();
                    response.push_str(&data);
                }
            }
        }
        OptionSerializer::None | OptionSerializer::Skip => {}
    }

    Ok(Some(response))
}
