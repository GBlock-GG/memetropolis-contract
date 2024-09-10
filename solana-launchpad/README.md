# Launchpad Anchor Project

This project is a Solana program developed using the Anchor framework. The following guide will help you set up the environment, run the tests, and deploy the program.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Building the Program](#building-the-program)
4. [Running Tests](#running-tests)
5. [Deploying the Program](#deploying-the-program)

## Prerequisites
Anchor Install
https://www.anchor-lang.com/docs/installation

## Installation
1. Install dependencies:
Install the necessary Rust and JavaScript dependencies:
```sh
anchor install
```

## Building the Program
Build the Anchor program:
```sh
anchor build
```

## Running Tests
1. Start the local Solana validator:
```sh
solana-test-validator
```
2. Run the tests:
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


