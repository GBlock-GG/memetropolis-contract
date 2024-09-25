// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Script.sol";
import "src/TokenFactory.sol";
import "src/Token.sol";

contract DeployTokenFactory is Script {
    function run() external {
        // Start broadcasting the deployment transaction
        vm.startBroadcast();

        uint MEMECOIN_FUNDING_GOAL = 20 ether;
        uint TOKEN_CREATOR_BONUS = 0.12 ether;
        uint PLATFORM_FEE = 0.6 ether;

        TokenFactory factory = new TokenFactory(
            msg.sender,
            MEMECOIN_FUNDING_GOAL,
            TOKEN_CREATOR_BONUS,
            PLATFORM_FEE,
            msg.sender
        );

        // Stop broadcasting the transaction
        vm.stopBroadcast();

        // Log the proxy contract address
        console.log("TokenFactory deployed at:", address(factory));
    }
}
