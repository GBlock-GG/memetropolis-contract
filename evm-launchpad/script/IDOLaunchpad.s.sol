// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {IDOLaunchpad} from "../src/IDOLaunchpad.sol";

contract IDOLaunchpadScript is Script {
    IDOLaunchpad public idoLaunchpad;

    function setUp() public {}

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);        
        //
        address memeCoinToken = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2; //WETH
        uint256 startTime = block.timestamp;
        uint256 endTime = block.timestamp + 7 days;
        uint256 tokenPrice = 1 ether;
        uint256 minInvestment = 0.1 ether;
        uint256 maxInvestment = 10 ether;
        idoLaunchpad = new IDOLaunchpad(
            memeCoinToken,
            address(0), //paymentToken
            tokenPrice,
            minInvestment,
            maxInvestment,
            startTime,
            endTime
        );
        vm.stopBroadcast();
    }
}
