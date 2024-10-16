// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Script.sol";
import "src/TokenFactory.sol";
import "src/Token.sol";

contract DeployTokenFactory is Script {
    address constant LZ_ENDPOINT_V2_ADDRESS = 0x6EDCE65403992e310A62460808c4b910D972f10f; // 0x1a44076050125825900e736c501f859c50fE728c

    function run() external {
        // Start broadcasting the deployment transaction
        vm.startBroadcast();

        uint TOKEN_CREATOR_BONUS = 0.12 ether;
        uint PLATFORM_FEE = 0.6 ether;
        uint INITIAL_PRICE = 2 * 10 ** 12;

        TokenFactory factory = new TokenFactory(
            msg.sender,
            TOKEN_CREATOR_BONUS,
            PLATFORM_FEE,
            LZ_ENDPOINT_V2_ADDRESS,
            INITIAL_PRICE
        );

        // Stop broadcasting the transaction
        vm.stopBroadcast();

        // Log the proxy contract address
        console.log("TokenFactory deployed at:", address(factory));
    }
}
