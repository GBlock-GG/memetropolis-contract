// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {TokenFactory} from "../src/TokenFactory.sol";
import {Token} from "../src/Token.sol";

contract TokenFactoryTest is Test {
    TokenFactory public factory;

    function setUp() public {
        factory = new TokenFactory();
    }

    function test_CreateToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        assertEq(token.balanceOf(address(factory)), factory.INIT_SUPPLY());
    }

    function test_BuyMemeToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        
        uint256 ethAmount = 40 ether;
        factory.buyMemeToken{value: ethAmount}(tokenAddress, 800000);
    }
}
