// SPDX-License-Identifier: Apache-2.0

use {
    crate::utils::{construct_instruction_accounts, construct_instruction_data, idl_from_json},
    anchor_syn::idl::{Idl, IdlInstruction},
    anyhow::{format_err, Result},
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::Keypair,
        signature::Signature,
        signature::Signer,
        signer::keypair::read_keypair_file,
        transaction::Transaction,
    },
    std::{ffi::OsStr, marker::PhantomData, str::FromStr},
};

/// Represents a Solana program call configuration and execution context.
///
/// This struct encapsulates the necessary data and parameters required to configure and execute a
/// call to a Solana program. It includes information such as the Solana RPC client, the Idl (Interface
/// Definition Language) for the program, the program's ID, the specific instruction to call, call data,
/// accounts involved, signers, new accounts to be created, and the payer's keypair.
pub struct SolanaTransaction {
    rpc_client: RpcClient,
    idl: Idl,
    program_id: Pubkey,
    instruction: IdlInstruction,
    call_data: Vec<u8>,
    accounts: Vec<AccountMeta>,
    signers: Vec<Keypair>,
    new_accounts: Vec<(Pubkey, String)>,
    payer: Keypair,
}

/// Type state for the call command to tell that some mandatory state has not yet
/// been set yet or to fail upon setting the same state multiple times.
pub struct Missing<S>(PhantomData<fn() -> S>);

pub mod state {
    //! Type states that tell what state of the command has not
    //! yet been set properly for a valid construction.

    /// Type state for the RPC client.
    pub struct RpcClient;
    /// Type state for the Idl.
    pub struct Idl;
    /// Type state for the program ID.
    pub struct ProgramID;
    /// Type state for the instruction.
    pub struct Instruction;
    /// Type state for the instruction data.
    pub struct CallData;
    /// Type state for the accounts.
    pub struct Accounts;
    /// Type state for the payer.
    pub struct Payer;
}

/// Represents options for configuring a Solana program call.
///
/// This struct is designed to be used with the [`SolanaTransactionBuilder`] to collect user-provided
/// configuration options for setting up a Solana program call. The options are stored as strings,
/// making it convenient for users to specify their desired parameters.
struct SolanaTransactionOpts {
    rpc_url: String,
    idl: String,
    program_id: String,
    instruction: String,
    call_data: Vec<String>,
    accounts: Vec<String>,
    payer: String,
}

/// A builder for configuring and constructing Solana program calls.
///
/// The [`SolanaTransactionBuilder`] allows you to fluently specify various configuration options for
/// setting up a Solana program call. These options include the Solana RPC client, Idl (Interface
/// Definition Language), program ID, instruction, call data, accounts, and payer. Once all
/// necessary parameters are set, you can use this builder to build a [`SolanaTransaction`] instance for
/// executing the program call.
#[allow(clippy::type_complexity)]
pub struct SolanaTransactionBuilder<
    RpcClient,
    Idl,
    ProgramID,
    Instruction,
    CallData,
    Accounts,
    Payer,
> {
    opts: SolanaTransactionOpts,
    marker: PhantomData<
        fn() -> (
            RpcClient,
            Idl,
            ProgramID,
            Instruction,
            CallData,
            Accounts,
            Payer,
        ),
    >,
}

impl Default
    for SolanaTransactionBuilder<
        Missing<state::RpcClient>,
        Missing<state::Idl>,
        Missing<state::ProgramID>,
        Missing<state::Instruction>,
        Missing<state::CallData>,
        Missing<state::Accounts>,
        Missing<state::Payer>,
    >
{
    fn default() -> Self {
        Self::new()
    }
}

impl
    SolanaTransactionBuilder<
        Missing<state::RpcClient>,
        Missing<state::Idl>,
        Missing<state::ProgramID>,
        Missing<state::Instruction>,
        Missing<state::CallData>,
        Missing<state::Accounts>,
        Missing<state::Payer>,
    >
{
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            opts: SolanaTransactionOpts {
                rpc_url: "".to_string(),
                idl: "".to_string(),
                program_id: "".to_string(),
                instruction: "".to_string(),
                call_data: vec![],
                accounts: vec![],
                payer: "".to_string(),
            },
            marker: PhantomData,
        }
    }
}

impl<Id, Pi, In, C, A, Py>
    SolanaTransactionBuilder<Missing<state::RpcClient>, Id, Pi, In, C, A, Py>
{
    /// Sets the Solana Remote Procedure Call (RPC) URL for connecting to a Solana network.
    ///
    /// The RPC URL is crucial for establishing a connection to a Solana cluster. It defines the
    /// endpoint through which the Solana program call will interact with the network. By specifying
    /// the RPC URL, you ensure that the program call is directed to the desired Solana cluster.
    ///
    /// # Parameters
    ///
    /// - `rpc_url`: A `String` representing the URL of the Solana RPC endpoint.
    ///
    /// # Returns
    ///
    /// Returns a new [`SolanaTransactionBuilder`] instance with the RPC URL option set.
    pub fn rpc_url<T: Into<String>>(
        self,
        rpc_url: T,
    ) -> SolanaTransactionBuilder<state::RpcClient, Id, Pi, In, C, A, Py> {
        SolanaTransactionBuilder {
            opts: SolanaTransactionOpts {
                rpc_url: rpc_url.into(),
                ..self.opts
            },
            marker: PhantomData,
        }
    }
}

impl<Rp, Pi, In, C, A, Py> SolanaTransactionBuilder<Rp, Missing<state::Idl>, Pi, In, C, A, Py> {
    /// Sets the Interface Definition Language (Idl) by providing the path to the Idl JSON file
    /// generated by the Solang compiler.
    ///
    /// The Idl JSON file is essential for defining the structure and interface of the Solana
    /// program being called. By specifying the path to this file, you ensure that the program call
    /// is configured with the correct Idl.
    ///
    /// # Parameters
    ///
    /// - `idl`: A `String` representing the path to the Idl JSON file.
    ///
    /// # Returns
    ///
    /// Returns a new [`SolanaTransactionBuilder`] instance with the Idl option set.
    pub fn idl<T: Into<String>>(
        self,
        idl: T,
    ) -> SolanaTransactionBuilder<Rp, state::Idl, Pi, In, C, A, Py> {
        SolanaTransactionBuilder {
            opts: SolanaTransactionOpts {
                idl: idl.into(),
                ..self.opts
            },
            marker: PhantomData,
        }
    }
}

impl<Rp, Id, In, C, A, Py>
    SolanaTransactionBuilder<Rp, Id, Missing<state::ProgramID>, In, C, A, Py>
{
    /// Sets the program ID for the Solana program call.
    ///
    /// The program ID is a unique identifier for a Solana program on the Solana blockchain. It
    /// specifies which program should be invoked when the Solana transaction is processed.
    ///
    /// # Parameters
    ///
    /// - `program_id`: A `String` representing the program ID for the Solana program.
    ///
    /// # Returns
    ///
    /// Returns a new [`SolanaTransactionBuilder`] instance with the program ID option set.
    pub fn program_id<T: Into<String>>(
        self,
        program_id: T,
    ) -> SolanaTransactionBuilder<Rp, Id, state::ProgramID, In, C, A, Py> {
        SolanaTransactionBuilder {
            opts: SolanaTransactionOpts {
                program_id: program_id.into(),
                ..self.opts
            },
            marker: PhantomData,
        }
    }
}

impl<Rp, Id, Pi, C, A, Py>
    SolanaTransactionBuilder<Rp, Id, Pi, Missing<state::Instruction>, C, A, Py>
{
    /// Sets the Solana program instruction to be called.
    ///
    /// An instruction represents a specific action or operation that a Solana program can perform.
    ///
    /// # Parameters
    ///
    /// - `instruction`: A `String` representing the name of the Solana program instruction.
    ///
    /// # Returns
    ///
    /// Returns a new [`SolanaTransactionBuilder`] instance with the specified instruction set.
    pub fn instruction<T: Into<String>>(
        self,
        instruction: T,
    ) -> SolanaTransactionBuilder<Rp, Id, Pi, state::Instruction, C, A, Py> {
        SolanaTransactionBuilder {
            opts: SolanaTransactionOpts {
                instruction: instruction.into(),
                ..self.opts
            },
            marker: PhantomData,
        }
    }
}

impl<Rp, Id, Pi, In, A, Py>
    SolanaTransactionBuilder<Rp, Id, Pi, In, Missing<state::CallData>, A, Py>
{
    /// Sets the call data for the Solana program instruction.
    ///
    /// Call data is essential for providing input parameters to the Solana program instruction.
    /// For each argument in the instruction's Idl (Interface Definition Language), there should be a corresponding string
    /// in the `call_data` vector. The format of these strings depends on the argument type:
    ///
    /// - For primitive types, such as integers or strings, a single string should be provided.
    /// - For structs, the string should be a JSON representation of the struct.
    /// - For arrays and vectors, the string should contain a comma-separated list of values.
    ///
    /// These data values will be compared to the Idl instruction's argument types and encoded accordingly
    /// when invoking the Solana program.
    ///
    /// # Parameters
    ///
    /// - `call_data`: A `Vec<String>` containing the call data for the Solana program instruction.
    ///
    /// # Returns
    ///
    /// Returns a new [`SolanaTransactionBuilder`] instance with the specified call data set.
    pub fn call_data<S: Into<String>>(
        self,
        call_data: Vec<S>,
    ) -> SolanaTransactionBuilder<Rp, Id, Pi, In, state::CallData, A, Py> {
        SolanaTransactionBuilder {
            opts: SolanaTransactionOpts {
                call_data: call_data.into_iter().map(|s| s.into()).collect(),
                ..self.opts
            },
            marker: PhantomData,
        }
    }
}

impl<Rp, Id, Pi, In, C, Py>
    SolanaTransactionBuilder<Rp, Id, Pi, In, C, Missing<state::Accounts>, Py>
{
    /// Sets the accounts for the Solana program instruction.
    ///
    /// Accounts represent the addresses associated with the Solana program instruction and can include
    /// user-defined accounts, system accounts, and special keywords. The provided `accounts` vector
    /// should contain strings corresponding to these accounts.
    ///
    /// You can use special keywords for some accounts:
    ///
    /// - `"new"`: Creates a new account and writes the keypair to a file. Information about the new account
    /// will be stored in the `new_accounts` field and can be accessed later.
    ///
    /// - `"self"`: Reads the default keypair from the local configuration file. This is useful for accessing
    /// the current user's account without specifying a keypair.
    ///
    /// - `"system"`: Represents the system program account.
    ///
    /// Whether an account is signable and mutable will be determined based on the account's definition in the
    /// Idl (Interface Definition Language). Accounts marked as signable in the Idl will be treated as signers,
    /// and mutable accounts will be set as mutable.
    ///
    /// # Parameters
    ///
    /// - `accounts`: A `Vec<String>` containing the account names or keywords.
    ///
    /// # Returns
    ///
    /// Returns a new [`SolanaTransactionBuilder`] instance with the specified accounts set.
    pub fn accounts<S: Into<String>>(
        self,
        accounts: Vec<S>,
    ) -> SolanaTransactionBuilder<Rp, Id, Pi, In, C, state::Accounts, Py> {
        SolanaTransactionBuilder {
            opts: SolanaTransactionOpts {
                accounts: accounts.into_iter().map(|s| s.into()).collect(),
                ..self.opts
            },
            marker: PhantomData,
        }
    }
}

impl<Rp, Id, Pi, In, C, A> SolanaTransactionBuilder<Rp, Id, Pi, In, C, A, Missing<state::Payer>> {
    /// Sets the payer for the Solana program instruction.
    ///
    /// The payer is the account responsible for covering the transaction fees associated with
    /// executing the Solana program instruction. It should be specified as the path to a keypair file.
    ///
    /// # Parameters
    ///
    /// - `payer`: A `String` containing the path to the keypair file for the payer account.
    ///
    /// # Returns
    ///
    /// Returns a new [`SolanaTransactionBuilder`] instance with the specified payer set.    
    pub fn payer<T: Into<String>>(
        self,
        payer: T,
    ) -> SolanaTransactionBuilder<Rp, Id, Pi, In, C, A, state::Payer> {
        SolanaTransactionBuilder {
            opts: SolanaTransactionOpts {
                payer: payer.into(),
                ..self.opts
            },
            marker: PhantomData,
        }
    }
}

impl
    SolanaTransactionBuilder<
        state::RpcClient,
        state::Idl,
        state::ProgramID,
        state::Instruction,
        state::CallData,
        state::Accounts,
        state::Payer,
    >
{
    /// Finalizes the configuration and prepares the [`SolanaTransaction`] instance for execution.
    ///
    /// This method initializes the [`SolanaTransaction`] instance with the provided configuration,
    /// including the RPC client, Idl, program ID, instruction, call data, accounts, signers,
    /// new accounts, and payer. If any errors occur during the configuration process,
    /// they will be returned as an `Err` variant of the `Result`.
    ///
    /// # Errors
    ///
    /// This method returns an error if any of the following conditions are met:
    ///
    /// - The Idl cannot be obtained from the specified JSON file.
    /// - The program ID cannot be parsed from the provided string.
    /// - The specified instruction is not found in the Idl.
    /// - There is an error constructing the call data.
    /// - There is an error constructing the accounts.
    /// - The payer keypair cannot be read from the specified file.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the configured [`SolanaTransaction`] instance if the configuration
    pub fn done(self) -> Result<SolanaTransaction> {
        // Get the RPC client
        let rpc_client = RpcClient::new_with_commitment(
            self.opts.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        // Get the Idl
        let idl = idl_from_json(OsStr::new(&self.opts.idl))
            .map_err(|e| format_err!("Error getting Idl from JSON file: {}", e))?;

        // Get the program ID
        let program_id = Pubkey::from_str(&self.opts.program_id)
            .map_err(|e| format_err!("Error getting program ID: {}", e))?;

        // Find the instruction with the specified name
        let instruction = idl
            .instructions
            .iter()
            .find(|i| i.name == self.opts.instruction)
            .ok_or_else(|| format_err!("Instruction {} not found", self.opts.instruction))?
            .clone();

        // Prepare the call data
        let idl_defined_types = idl.types.clone();
        let call_data =
            construct_instruction_data(&instruction, &self.opts.call_data, &idl_defined_types)
                .map_err(|e| format_err!("Error constructing call data: {}", e))?;

        // Prepare the accounts
        let (accounts, signers, new_accounts) =
            construct_instruction_accounts(&instruction, &self.opts.accounts)
                .map_err(|e| format_err!("Error constructing accounts: {}", e))?;

        // Get the payer
        let payer = read_keypair_file(&self.opts.payer)
            .map_err(|e| format_err!("Error getting payer: {}", e))?;

        Ok(SolanaTransaction {
            rpc_client,
            idl,
            program_id,
            instruction,
            call_data,
            accounts,
            signers,
            new_accounts,
            payer,
        })
    }
}

#[allow(clippy::new_ret_no_self)]
impl SolanaTransaction {
    /// Returns a clean builder for [`SolanaTransaction`]
    #[allow(clippy::type_complexity)]
    pub fn new() -> SolanaTransactionBuilder<
        Missing<state::RpcClient>,
        Missing<state::Idl>,
        Missing<state::ProgramID>,
        Missing<state::Instruction>,
        Missing<state::CallData>,
        Missing<state::Accounts>,
        Missing<state::Payer>,
    > {
        SolanaTransactionBuilder::new()
    }

    /// Get the Rpc client
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Get the Idl
    pub fn idl(&self) -> &Idl {
        &self.idl
    }

    /// Get the program ID
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    /// Get the instruction
    pub fn instruction(&self) -> &IdlInstruction {
        &self.instruction
    }

    /// Get the call data
    pub fn call_data(&self) -> &Vec<u8> {
        &self.call_data
    }

    /// Get the accounts
    pub fn accounts(&self) -> &Vec<AccountMeta> {
        &self.accounts
    }

    /// Get the signers
    pub fn signers(&self) -> &Vec<Keypair> {
        &self.signers
    }

    /// Get the new accounts
    pub fn new_accounts(&self) -> &Vec<(Pubkey, String)> {
        &self.new_accounts
    }

    /// Get the payer
    pub fn payer(&self) -> &Keypair {
        &self.payer
    }

    /// Submits a transaction to the Solana network using the configured parameters.
    ///
    /// This method prepares and submits a transaction to the Solana network based on the
    /// previously configured parameters, such as the program ID, instruction, call data,
    /// accounts, signers, and payer. It handles the construction and signing of the transaction,
    /// retrieves the latest blockhash from the RPC server, and sends the transaction to the network.
    ///
    /// # Errors
    ///
    /// This method returns an error if any of the following conditions are met:
    ///
    /// - The RPC client encounters an error when fetching the latest blockhash.
    /// - Signing the transaction with the payer or other signers fails.
    /// - Sending and confirming the transaction on the Solana network fails.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the transaction's [`Signature`] if the submission process succeeds.
    pub fn submit_transaction(&self) -> Result<Signature> {
        // Create the instruction
        let instruction = Instruction {
            program_id: self.program_id,
            accounts: self.accounts.clone(),
            data: self.call_data.clone(),
        };

        // Create the message
        let payer_keypair = &self.payer;
        let message = Message::new(&[instruction], Some(&payer_keypair.pubkey()));
        let mut transaction = Transaction::new_unsigned(message);

        let rpc_client = &self.rpc_client;
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .map_err(|err| format_err!("error: unable to get latest blockhash: {}", err))?;

        // The payer needs to sign the transaction.
        // This method does not require all keypairs to be provided.
        // Note: It is permitted to sign a transaction with the same keypair multiple times.
        transaction.partial_sign(&[payer_keypair], recent_blockhash);

        let signers = self
            .signers
            .iter()
            .map(|keypair| keypair as &dyn Signer)
            .collect::<Vec<&dyn Signer>>();

        // Sign the transaction
        transaction
            .try_sign(&signers, recent_blockhash)
            .map_err(|err| format_err!("error: failed to sign transaction: {}", err))?;

        let signature = rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .map_err(|err| format_err!("Error: {}", err,))?;

        Ok(signature)
    }
}
