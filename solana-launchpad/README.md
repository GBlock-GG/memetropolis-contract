# Launchpad Anchor Project

This project is a Solana program developed using the Anchor framework. The following guide will help you set up the environment, run the tests, and deploy the program.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building the Program](#building-the-program)
3. [Running Tests](#running-tests)
4. [Deploying the Program](#deploying-the-program)

## Prerequisites
OS: Ubuntu 22

Rust Install
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Solana Install
```sh
sh -c "$(curl -sSfL https://release.solana.com/v1.18.18/install)"
```
Key Generation For testing
```sh
solana-keygen new
```

Node.js & Yarn Install

Anchor Install
```sh
cargo install --git https://github.com/coral-xyz/anchor --tag v0.29.0 anchor-cli
```

## Building the Program
```sh
anchor build
```

## Running Tests
Run the tests:
```sh
anchor test
```

## Deploying the Program
Once the tests pass and you're ready to deploy to the devnet, follow these steps:
1. Set the Solana CLI to use the devnet:
```sh
solana config set --url https://api.devnet.solana.com
```
2. Deploy the program:
```sh
anchor deploy
```
This command will deploy your program to the devnet and update the program ID in the Anchor.toml file.


