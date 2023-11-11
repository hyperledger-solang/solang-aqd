// SPDX-License-Identifier: Apache-2.0

use clap::{Parser, Subcommand};

#[cfg(feature = "solana")]
use aqd_solana::SolanaAction;

#[cfg(feature = "polkadot")]
use aqd_polkadot::PolkadotAction;

#[derive(Parser)]
#[command(  author = env!("CARGO_PKG_AUTHORS"), 
            about = "Aqd is a versatile CLI tool for interacting with contracts on Solana and Polkadot blockchains.", 
            subcommand_required = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
pub enum Commands {
    #[cfg(feature = "solana")]
    #[command(about = "Interact with Solana contracts on chain")]
    Solana {
        #[clap(subcommand)]
        action: SolanaAction,
    },
    #[cfg(feature = "polkadot")]
    #[command(about = "Interact with Polkadot contracts on chain")]
    Polkadot {
        #[clap(subcommand)]
        action: PolkadotAction,
    },
}
