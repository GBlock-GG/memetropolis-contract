import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import chain from "./chain.json";
const config: HardhatUserConfig = {
  solidity: "0.8.24",
  networks: {
    sepolia: {
      url: chain.sepolia.url,
      accounts: [chain.sepolia.pk],
    }
  },
  etherscan:{
    apiKey:"H8EZ3CUEI77YQG5G83EGCQSQ736W5ZXSZ4",
  }
};

export default config;
