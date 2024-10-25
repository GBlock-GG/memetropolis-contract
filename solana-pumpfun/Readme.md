## build & test

Run solana local validator

```bash
solana-test-validator --clone-upgradeable-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s --url https://api.mainnet-beta.solana.com --reset
```

In other terminal, Run anchor

```bash
anchor test --skip-local-validator
```
