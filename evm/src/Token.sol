// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract Token is ERC20, Ownable {
    address public tokenFactoryAddress;
    constructor(
        string memory name,
        string memory symbol,
        uint256 initialMintValue,
        address owner
    ) ERC20(name, symbol) Ownable(owner) {
        _mint(msg.sender, initialMintValue);
        tokenFactoryAddress = msg.sender;
    }
}
