import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const MyOFTModule = buildModule("MyOFT", (m) => {
  const tokenName = m.getParameter("tokenName");
  const tokenSymbol = m.getParameter("tokenSymbol");
  const totalSupply = m.getParameter("totalSupply");
  const endpoint = m.getParameter("endpoint");
  const owner = m.getParameter("owner");
  //deploy contract
  const myOft = m.contract("MyOFT", [tokenName, tokenSymbol, totalSupply, endpoint, owner]);
  return { myOft };
});

export default MyOFTModule;
