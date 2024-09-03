// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {IDOLaunchpad} from "../src/IDOLaunchpad.sol";

contract IDOLaunchpadScript is Script {
    IDOLaunchpad public idoLaunchpad;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        // idoLaunchpad = new IDOLaunchpad();

        vm.stopBroadcast();
    }
}
