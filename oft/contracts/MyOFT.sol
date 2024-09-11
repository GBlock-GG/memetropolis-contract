// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {OFT} from "@layerzerolabs/lz-evm-oapp-v2/contracts/oft/OFT.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract MyOFT is OFT {
    uint8 private _decimals;
    constructor(
        string memory _tokenName,
        string memory _tokenSymbol,
        uint256 _totalSupply,
        address _layerZeroEndpoint,
        address _owner
    ) OFT(_tokenName, _tokenSymbol, _layerZeroEndpoint, _owner) Ownable(_owner) {
        _credit(_owner, _totalSupply, 0);
    }
}