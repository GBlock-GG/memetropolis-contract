// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "../library/ABDKMathQuad.sol";

contract MathLibraryTest {
    function uint256ToBytes16(uint256 value) public pure returns (bytes16) {
        require(value <= type(uint128).max, "Value too large for bytes16");
        return bytes16(uint128(value));
    }

    function bytes16ToUint256(bytes16 value) public pure returns (uint256) {
        return uint256(uint128(value));
    }

    function expFunc(uint256 x) pure public returns (uint256 exp) {
        bytes16 tX = ABDKMathQuad.fromUInt(x);
        bytes16 tExp = ABDKMathQuad.exp(tX);
        exp = ABDKMathQuad.toUInt(tExp);
    }

    function lnFunc(uint256 x) pure public returns (uint256 ln) {
        bytes16 tX = ABDKMathQuad.fromUInt(x);
        bytes16 tLn = ABDKMathQuad.ln(tX);
        ln = ABDKMathQuad.toUInt(tLn);
    }
}
