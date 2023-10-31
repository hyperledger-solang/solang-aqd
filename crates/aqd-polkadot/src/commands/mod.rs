// SPDX-License-Identifier: Apache-2.0

mod call;
mod instantiate;
mod remove;
mod upload;

pub use self::{
    call::PolkadotCallCommand, instantiate::PolkadotInstantiateCommand,
    remove::PolkadotRemoveCommand, upload::PolkadotUploadCommand,
};

use {std::path::PathBuf, url::Url};

pub use contract_extrinsics::BalanceVariant;

/// Common CLI options for executing extrinsics on a Polkadot node.
///
/// These options allow you to specify the contract or metadata file, the node's URL,
/// network, secret key URI, whether to execute the extrinsic, the storage deposit limit,
/// and whether to export the output in JSON format.
#[derive(Clone, Debug, clap::Args)]
pub struct CLIExtrinsicOpts {
    #[clap(
        value_parser,
        help = "Specifies the path to a contract wasm file, .contract bundle, or .json metadata file."
    )]
    file: PathBuf,
    #[clap(
        name = "url",
        long,
        value_parser,
        default_value = "ws://localhost:9944",
        help = "Specifies the websockets URL for the substrate node directly."
    )]
    url: Url,
    #[clap(
        value_enum,
        name = "network",
        long,
        conflicts_with = "url",
        help = "Specifies the network name."
    )]
    network: Option<Network>,
    #[clap(
        name = "suri",
        long,
        short,
        help = "Specifies the secret key URI used for deploying the contract. For example:\n
    For a development account: //Alice\n
    With a password: //Alice///SECRET_PASSWORD"
    )]
    suri: String,
    #[clap(
        short('x'),
        long,
        help = "Specifies whether to submit the extrinsic for execution."
    )]
    execute: bool,
    #[clap(
        long,
        help = "Specifies the maximum amount of balance that can be charged from the caller to pay for the storage consumed."
    )]
    storage_deposit_limit: Option<BalanceVariant>,
    #[clap(long, help = "Specifies whether to export the call output in JSON.")]
    output_json: bool,
}

/// Available networks.
#[derive(clap::ValueEnum, Clone, Debug)]
enum Network {
    Rococo,
    PhalaPoC5,
    AstarShiden,
    AstarShibuya,
    Astar,
    AlephZeroTestnet,
    AlephZero,
    T3RNT0RN,
    PendulumTestnet,
}

impl CLIExtrinsicOpts {
    /// Returns the URL for the Polkadot node based on the specified network or user input.
    ///
    /// If a specific network is chosen, the function returns the URL associated with that network.
    /// Otherwise, it returns the URL provided by the user in the CLI options.
    pub fn url(&self) -> Url {
        if let Some(net) = &self.network {
            match net {
                Network::Rococo => Url::parse("wss://rococo-contracts-rpc.polkadot.io").unwrap(),
                Network::PhalaPoC5 => Url::parse("wss://poc5.phala.network/ws").unwrap(),
                Network::AstarShiden => Url::parse("wss://rpc.shiden.astar.network").unwrap(),
                Network::AstarShibuya => Url::parse("wss://rpc.shibuya.astar.network").unwrap(),
                Network::Astar => Url::parse("wss://rpc.astar.network").unwrap(),
                Network::AlephZeroTestnet => Url::parse("wss://ws.test.azero.dev").unwrap(),
                Network::AlephZero => Url::parse("wss://ws.azero.dev").unwrap(),
                Network::T3RNT0RN => Url::parse("wss://ws.t0rn.io").unwrap(),
                Network::PendulumTestnet => {
                    Url::parse("wss://rpc-foucoco.pendulumchain.tech").unwrap()
                }
            }
        } else {
            self.url.clone()
        }
    }
}
