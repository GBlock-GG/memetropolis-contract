// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

library LogExpMath {
    int128 private constant ln2 = 0xB17217F7D1CF79AB;
    uint256 private constant MAX_X = 9223372036854775807 * 1e18;

    /// @notice Helper function to approximate e^x using a Taylor series expansion.
    /// @param x The exponent value.
    /// @return The approximated result of e^x.
    function exp(uint256 x) external pure returns (uint256) {
        uint256 sum = 10**18;  // Start with 1 * 10^18 for precision
        uint256 term = 10**18;  // Initial term = 1 * 10^18
        uint256 xPower = x;  // Initial power of x
        
        for (uint256 i = 1; i <= 20; i++) {  // Increase iterations for better accuracy
            term = (term * xPower) / (i * 10**18);  // x^i / i!
            sum += term;

            // Prevent overflow and unnecessary calculations
            if (term < 1) break;
        }

        return sum;
    }

    function fromInt128(int128 x) internal pure returns (uint256) {
        // Multiply by 1e18 and shift right by 64 bits
        return (uint256(int256(x)) * 1e18) / (1 << 64);
    }

    function toInt128(uint256 x) internal pure returns (int128) {
        require(x <= MAX_X, "BondingCurve: Overflow in toInt128");
        return int128(int256((x << 64) / 1e18));
    }

        // Helper function to find most significant bit
    function log_2(uint256 x) internal pure returns (int128) {
        require(x > 0, "BondingCurve: Input must be greater than zero");

        int128 x64x64 = toInt128(x);

        int256 msb = 0;
        uint256 xc = uint256(int256(x64x64));
        if (xc >= 0x10000000000000000) {
            xc >>= 64;
            msb += 64;
        }
        if (xc >= 0x100000000) {
            xc >>= 32;
            msb += 32;
        }
        if (xc >= 0x10000) {
            xc >>= 16;
            msb += 16;
        }
        if (xc >= 0x100) {
            xc >>= 8;
            msb += 8;
        }
        if (xc >= 0x10) {
            xc >>= 4;
            msb += 4;
        }
        if (xc >= 0x4) {
            xc >>= 2;
            msb += 2;
        }
        if (xc >= 0x2) msb += 1; // No need to shift xc anymore

        int256 result = (msb - 64) << 64;

        uint256 ux = uint256(int256(x64x64)) << uint256(127 - msb);

        for (int256 bit = 0x8000000000000000; bit > 0; bit >>= 1) {
            ux = ux * ux;
            uint256 bbit = ux >> 255;
            ux >>= 127 + bbit;
            result += bit * int256(bbit);
        }

        return int128(result);
    }

    function ln(uint256 x) external pure returns (uint256) {
        require(x > 0, "BondingCurve: Input must be greater than zero");

        int128 log2Result = log_2(x); // log_2 in 64.64 format

        // Multiply and adjust back to 64.64 format
        int128 lnResult = int128((int256(log2Result) * int256(ln2)) >> 64);

        return fromInt128(lnResult);
    }
}
