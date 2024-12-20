// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "./LogExpMath.sol";

library TokenFactoryHelper {
    uint256 private constant DECIMALS = 10 ** 18;

    function decodeMessage(bytes memory message) external pure returns (uint8 msgType, bytes32 memeTokenAddressBytes, bytes32 recipientAddressBytes, uint256 ethAmount, uint256 tokenQty) {
        uint256 index = 0;

        // Decode the msgType (BUY_TYPE or SELL_TYPE) from the first byte
        msgType = uint8(message[index]);
        index += 1;

        // Use assembly to load 32 bytes from memory starting at position `index`
        assembly {
            memeTokenAddressBytes := mload(add(message, add(0x20, index)))
        }
        index += 32;

        // Decode recipientAddressBytes using assembly
        assembly {
            recipientAddressBytes := mload(add(message, add(0x20, index)))
        }
        index += 32;

        // Decode ethAmount (uint128, 16 bytes), but load 32 bytes and cast to uint128
        uint128 ethAmount128;
        assembly {
            ethAmount128 := mload(add(message, add(0x10, index)))
        }
        ethAmount = uint256(ethAmount128); // Cast uint128 to uint256
        index += 16;

        // Decode tokenQty (uint256, 32 bytes)
        assembly {
            tokenQty := mload(add(message, add(0x20, index)))
        }
    }

    /// @notice Calculates the cost in wei for purchasing tokens using an exponential bonding curve.
    /// @return The total cost in wei required to purchase the tokens.
    function calculateCost(uint256 currentSupply, uint256 tokensToBuy, uint256 tokenK, uint256 tokenInitialPrice) external pure returns (uint256) {
        
        // Calculate the exponent parts scaled to avoid precision loss
        uint256 exponent1 = (tokenK * (currentSupply + tokensToBuy)) / DECIMALS;
        uint256 exponent2 = (tokenK * currentSupply) / DECIMALS;

        // Calculate e^(kx) using the exp function
        uint256 exp1 = LogExpMath.exp(exponent1);
        uint256 exp2 = LogExpMath.exp(exponent2);

        // Cost formula: (P0 / k) * (e^(k * (currentSupply + tokensToBuy)) - e^(k * currentSupply))
        // We use (P0 * 10^18) / k to keep the division safe from zero
        uint256 cost = (tokenInitialPrice * (exp1 - exp2)) / tokenK;  // Adjust for k scaling without dividing by zero
        return cost;
    }

    /// @notice Calculates the cost in wei for purchasing tokens using an exponential bonding curve.
    /// @return The tokens to purchase.
    function calculateTokenAmount(uint256 currentSupply, uint256 ethAmount, uint256 tokenK, uint256 tokenInitialPrice) external pure returns (uint256) {
        
        // Calculate e^(k * currentSupply)
        uint256 exp = LogExpMath.exp(tokenK * currentSupply / DECIMALS);

        // Calculate (ethAmount * K) / INIT_PRICE
        uint256 num = (ethAmount * tokenK) / tokenInitialPrice;

        // Calculate ln((ethAmount * K) / INIT_PRICE + e^(k * currentSupply))
        uint256 ln = LogExpMath.ln(num + exp);

        // formula: (ln((ethAmount * K) / INIT_PRICE + e^(k * currentSupply)) / K) - currentSupply
        uint256 tokenAmount = ln * DECIMALS / tokenK - currentSupply;  // Adjust for k scaling without dividing by zero
        return tokenAmount;
    }
}
