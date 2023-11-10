# Solang Aqd - Smart Contract CLI Tool

<div align="center">
  <img src="https://raw.githubusercontent.com/hyperledger/solang/main/docs/hl_solang_horizontal-color.svg" alt="Solang Logo" width="75%" />

  [![Discord](https://img.shields.io/discord/905194001349627914?logo=Hyperledger&style=plastic)](https://discord.gg/hyperledger)
  [![CI](https://img.shields.io/github/workflow/status/hyperledger/solang-aqd/CI?label=CI&logo=GitHub)](https://github.com/hyperledger/solang-aqd/actions)
  [![License](https://img.shields.io/github/license/hyperledger/solang.svg)](LICENSE)
</div>

`Aqd`(عَقد - meaning a contract in Arabic) is a versatile CLI tool for interacting with smart contracts on the Solana and Polkadot blockchains.
It provides a user-friendly interface with commands for deploying smart contracts and calling specific functions on the deployed contracts.

Whether you're developing on Solana or Polkadot, `Aqd` simplifies your smart contract interactions.

## Usage  
### Installation  
You can install Aqd using `cargo` :
```bash 
cargo install --force --locked aqd
```

### Polkadot Interactions
To upload a contract to Polkadot: 
```bash 
aqd polkadot upload --suri //Alice -x flipper.contract
```

To instantiate a contract on Polkadot:
```bash
aqd polkadot instantiate --suri //Alice --args true -x flipper.contract
```

To call a specific function on Polkadot: 
```bash
aqd polkadot call --contract <contract_address> --message get --suri //Alice flipper.contract
```
### Solana Interactions

To deploy a contract to Solana:
```bash
aqd solana deploy flipper.so
```

To call a specific function on Solana:
```bash
aqd solana call --idl flipper.json --program <program_id> --instruction new --data true --accounts new self system
```

For more information, refer to [`Solang Aqd` documentation](https://solang.readthedocs.io/en/v0.3.3/running.html)


## Packages

| Package                   | Description                                           | Version        |
| ------------------        | ----------------------------------                    | -------        |
| `aqd-core`                | The CLI tool core crate                               | pre-release    |
| `aqd-polkadot`            | Smart contract interactions for Polkadot              | pre-release    |
| `aqd-solana`              | Smart contract interactions for Solana                | pre-release    |
| `aqd-utils`               | Utility functions and common code                     | pre-release    |
| `aqd-solana-contracts`    | Rust crate for Solana smart contract interactions     | pre-release    |
