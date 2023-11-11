// SPDX-License-Identifier: Apache-2.0

mod cli;
use {
    crate::cli::{Cli, Commands::*},
    clap::{CommandFactory, FromArgMatches},
    std::process::exit,
};

#[cfg(feature = "solana")]
use aqd_solana::SolanaAction;

#[cfg(feature = "polkadot")]
use {aqd_polkadot::PolkadotAction, tokio::runtime::Runtime};

/// The main entry point for `aqd` command-line application.
fn main() {
    // Parse command-line arguments.
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap();

    #[cfg(feature = "polkadot")]
    let runtime = Runtime::new().expect("Failed to create Tokio runtime");

    match cli.command {
        #[cfg(feature = "solana")]
        Solana { action } => match action {
            SolanaAction::Deploy(deploy_args) => {
                if let Err(err) = deploy_args.handle() {
                    eprintln!("{}", err);
                    exit(1);
                }
            }
            SolanaAction::Call(call_args) => {
                if let Err(err) = call_args.handle() {
                    eprintln!("{}", err);
                    exit(1);
                }
            }
            SolanaAction::Show(show_args) => {
                if let Err(err) = show_args.handle() {
                    eprintln!("{}", err);
                    exit(1);
                }
            }
        },
        #[cfg(feature = "polkadot")]
        Polkadot { action } => match action {
            PolkadotAction::Upload(upload_args) => runtime.block_on(async {
                if let Err(err) = upload_args.handle().await {
                    eprintln!("{}", err);
                    exit(1);
                }
            }),
            PolkadotAction::Instantiate(instantiate_args) => runtime.block_on(async {
                if let Err(err) = instantiate_args.handle().await {
                    eprintln!("{}", err);
                    exit(1);
                }
            }),
            PolkadotAction::Call(call_args) => runtime.block_on(async {
                if let Err(err) = call_args.handle().await {
                    eprintln!("{}", err);
                    exit(1);
                }
            }),
            PolkadotAction::Remove(remove_args) => runtime.block_on(async {
                if let Err(err) = remove_args.handle().await {
                    eprintln!("{}", err);
                    exit(1);
                }
            }),
        },
    }
}
