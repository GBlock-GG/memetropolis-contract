# OFT Hardhat Project

## Install

```shell
> yarn
```

## Compile & Test contract

```shell
> yarn compile
```

```shell
> yarn test
````

## Deploy in Sepolia
1. Update "sepolia"/"pk" in ./chain.json
2. Update OFT info (tokenName, symbol, owner) in ./ignition/sepolia_params.json
   (the endpoint value is for sepolia, no update)
3. run command
```shell
> yarn deploy:sepolia
````