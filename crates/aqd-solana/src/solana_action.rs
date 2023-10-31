// SPDX-License-Identifier: Apache-2.0

use {
    crate::{SolanaCall, SolanaDeploy, SolanaShow},
    clap::Subcommand,
};

/// Available subcommands for the `solana` subcommand.
#[derive(Debug, Subcommand)]
pub enum SolanaAction {
    Deploy(SolanaDeploy),
    Call(SolanaCall),
    Show(SolanaShow),
}
