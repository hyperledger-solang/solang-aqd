// SPDX-License-Identifier: Apache-2.0

use {
    crate::{
        PolkadotCallCommand, PolkadotInstantiateCommand, PolkadotRemoveCommand,
        PolkadotUploadCommand,
    },
    clap::Subcommand,
};

/// Available subcommands for the `polkadot` subcommand.
#[derive(Debug, Subcommand)]
pub enum PolkadotAction {
    Upload(PolkadotUploadCommand),
    Instantiate(PolkadotInstantiateCommand),
    Call(PolkadotCallCommand),
    Remove(PolkadotRemoveCommand),
}
