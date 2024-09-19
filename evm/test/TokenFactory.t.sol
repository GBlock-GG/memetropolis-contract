// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {TokenFactory} from "../src/TokenFactory.sol";
import {Token} from "../src/Token.sol";

contract TokenFactoryTest is Test {
    TokenFactory public factory;
    uint constant DECIMALS = 10 ** 18;

    function setUp() public {
        factory = new TokenFactory(address(this));
    }

    function test_CreateToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        assertEq(token.balanceOf(address(factory)), factory.INIT_SUPPLY());
    }

    function test_BuyMemeToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy 20K tokens initially
        uint currentSupplyScaled = (token.totalSupply() - factory.INIT_SUPPLY()) / DECIMALS;
        uint tokenQty = 20000;
        uint requiredEth = factory.calculateCost(currentSupplyScaled, tokenQty);
        assertEq(requiredEth, tokenQty * factory.INITIAL_PRICE());
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty);
        assertEq(token.balanceOf(address(this)), tokenQty * DECIMALS);

        // Buy 10K tokens more
        currentSupplyScaled = (token.totalSupply() - factory.INIT_SUPPLY()) / DECIMALS;
        console.log("Current Supply Scaled: ", currentSupplyScaled);
        uint tokenQty2 = 10000;
        requiredEth = factory.calculateCost(currentSupplyScaled, tokenQty2);
        console.log("Required ETH: ", requiredEth);
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty2);
        assertEq(token.balanceOf(address(this)), (tokenQty + tokenQty2) * DECIMALS);
    }

    function test_SellMemeToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy 20K tokens
        uint currentSupplyScaled = (token.totalSupply() - factory.INIT_SUPPLY()) / DECIMALS;
        uint tokenQty = 20000;
        uint requiredEth = factory.calculateCost(currentSupplyScaled, tokenQty);
        assertEq(requiredEth, tokenQty * factory.INITIAL_PRICE());
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty);
        assertEq(token.balanceOf(address(this)), tokenQty * DECIMALS);

        // Sell 10K tokens
        uint sellTokenQty = 10000 * 10 ** 18;
        token.approve(address(factory), sellTokenQty);
        
        uint prevEthBalance = address(this).balance;
        factory.sellMemeToken(tokenAddress, sellTokenQty);
        uint increasedEthBalance = address(this).balance - prevEthBalance;

        assertEq(increasedEthBalance, factory.INITIAL_PRICE() * sellTokenQty / DECIMALS);
    }

    /// -- This test case needs mainnet fork to check
    function test_CreateLiquidityPool() public {
        uint depositAmount = 30 ether;
        vm.deal(address(this), depositAmount);

        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy 20K tokens initially
        uint currentSupplyScaled = (token.totalSupply() - factory.INIT_SUPPLY()) / DECIMALS;
        uint tokenQty = 800000;
        uint requiredEth = factory.calculateCost(currentSupplyScaled, tokenQty);
        assertEq(requiredEth, tokenQty * factory.INITIAL_PRICE());
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty);
        assertEq(token.balanceOf(address(this)), tokenQty * DECIMALS);
    }

    function testWithdrawETH() public {
        // Setup initial conditions
        uint depositAmount = 1 ether;
        vm.deal(address(this), depositAmount);
        (bool success, ) = payable(address(factory)).call{value: depositAmount}("");
        require(success, "Failed to send Ether");

        uint withdrawalAmount = 1 ether;
        factory.withdrawETH(withdrawalAmount);
        assertEq(address(this).balance, withdrawalAmount, "Withdraw ETH failed");
    }
    
    function testWithdrawToken() public {
        // Setup initial conditions
        uint initialMintValue = 2000 * 10 ** 18;
        Token newToken = new Token("A", "A", initialMintValue);
        newToken.transfer(address(factory), initialMintValue);

        factory.withdrawTokens(address(newToken), initialMintValue);
        assertEq(newToken.balanceOf(address(this)), initialMintValue, "Withdraw Token failed");
    }

    receive() external payable {}  // Allow the contract to receive ETH
}
