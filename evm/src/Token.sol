// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "@layerzerolabs/lz-evm-oapp-v2/contracts/oft/OFT.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract Token is OFT {
    address public tokenFactoryAddress;
    constructor(
        string memory name,
        string memory symbol,
        uint256 initialMintValue,
        address layerZeroEndpoint,
        address owner
    ) OFT(name, symbol, layerZeroEndpoint, owner) Ownable(owner) {
        _mint(msg.sender, initialMintValue);
        tokenFactoryAddress = msg.sender;
    }
}
