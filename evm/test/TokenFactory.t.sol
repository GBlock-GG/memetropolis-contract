// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import {console} from "forge-std/console.sol";
import {TokenFactory} from "../src/TokenFactory.sol";
import {Token} from "../src/Token.sol";
import {SimpleToken} from "../src/mock/SimpleToken.sol";

import { TestHelperOz5 } from "@layerzerolabs/test-devtools-evm-foundry/contracts/TestHelperOz5.sol";

contract TokenFactoryTest is TestHelperOz5 {
    TokenFactory public factory;
    TokenFactory public aFactory;
    TokenFactory public bFactory;
    uint constant DECIMALS = 10 ** 18;
    uint TOKEN_CREATOR_BONUS = 0.12 ether;
    uint PLATFORM_FEE = 0.6 ether;
    address constant LZ_ENDPOINT_V2_ADDRESS = 0x6EDCE65403992e310A62460808c4b910D972f10f; // 0x1a44076050125825900e736c501f859c50fE728c

    uint32 aEid = 1;
    uint32 bEid = 2;

    address public userA = address(0x1);

    function setUp() public override {
        vm.deal(userA, 1000 ether);

        super.setUp();
        setUpEndpoints(2, LibraryType.UltraLightNode);
    
        // factory = new TokenFactory(
        //     address(this),
        //     TOKEN_CREATOR_BONUS,
        //     PLATFORM_FEE,
        //     LZ_ENDPOINT_V2_ADDRESS
        // );

        aFactory = new TokenFactory(
            address(this),
            TOKEN_CREATOR_BONUS,
            PLATFORM_FEE,
            endpoints[1]
        );
        bFactory = new TokenFactory(
            address(this),
            TOKEN_CREATOR_BONUS,
            PLATFORM_FEE,
            endpoints[2]
        );
        aFactory.setPeer(bEid, bytes32(uint256(uint160(address(bFactory)))));
        bFactory.setPeer(aEid, bytes32(uint256(uint160(address(aFactory)))));
    }

    function test_BuyCrosschainMemetoken() public {
        vm.prank(userA);
        address tokenAddress = bFactory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        console.log("Init token balance: ", Token(tokenAddress).balanceOf(address(bFactory)));

        uint128 ethAmount = 10**15;
        (uint256 nativeFee, ) = aFactory.quoteBuyCrossChainMemetoken(2, tokenAddress, ethAmount);
        aFactory.buyCrosschainMemetoken{ value: nativeFee }(2, tokenAddress, ethAmount);

        // verify packet to bFactory manually
        verifyPackets(bEid, addressToBytes32(address(bFactory)));

        console.log("Current token balance", Token(tokenAddress).balanceOf(address(bFactory)));
    }

    function test_CreateToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);

        assertEq(token.balanceOf(address(factory)), factory.MAX_SUPPLY());
    }

    function test_BuyMemeToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy 20K tokens initially
        uint tokenQty = 20000 * 10 ** 18;
        uint requiredEth = factory.getRequiredEth(tokenAddress, tokenQty);
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty);
        assertEq(token.balanceOf(address(this)), tokenQty);

        // Buy 10K tokens more
        uint tokenQty2 = 10000 * 10 ** 18;
        requiredEth = factory.getRequiredEth(tokenAddress, tokenQty2);
        uint payEth = requiredEth + DECIMALS;
        
        uint originalEthBalance = address(this).balance;
        factory.buyMemeToken{value: payEth}(tokenAddress, tokenQty2);
        assertEq(originalEthBalance - address(this).balance, requiredEth);
        assertEq(token.balanceOf(address(this)), (tokenQty + tokenQty2));
    }


    function test_BuyMemeTokenInETH() public {
        uint depositAmount = 10 ether;
        vm.deal(address(this), depositAmount);
        
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy token equivalent to 1 ETH
        uint ethAmount = 1 ether;
        uint estimatedTokenOut = factory.getTokenOutOnBuy(tokenAddress, ethAmount);
        console.log("Estimated Token out amount: ", estimatedTokenOut);
        
        factory.buyMemeTokenInEth{value: ethAmount}(tokenAddress, 0);
        console.log("Real Token out amount: ", token.balanceOf(address(this)));
    }

    function test_BuyAllMemeToken() public {
        uint depositAmount = 30 ether;
        vm.deal(address(this), depositAmount);

        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy 800K tokens
        uint tokenQty = 800_000 * 10 ** 18;
        uint requiredEth = factory.getRequiredEth(tokenAddress, tokenQty);
        console.log("Buy All - Required ETH: ", requiredEth);
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty);
        assertEq(token.balanceOf(address(this)), tokenQty);

        // check current price
        uint tokenPrice = factory.getCurrentTokenPrice(tokenAddress);
        uint marketCap = tokenPrice * token.totalSupply() / DECIMALS;
        console.log("Buy All - Marketcap: ", marketCap);
    }

    function test_SellMemeToken() public {
        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy 20K tokens
        uint tokenQty = 20000 * 10 ** 18;
        uint requiredEth = factory.getRequiredEth(tokenAddress, tokenQty);
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty);
        assertEq(token.balanceOf(address(this)), tokenQty);

        // Sell 10K tokens
        uint sellTokenQty = 10000 * 10 ** 18;
        token.approve(address(factory), sellTokenQty);
        
        uint prevEthBalance = address(this).balance;
        factory.sellMemeToken(tokenAddress, sellTokenQty);
        uint increasedEthBalance = address(this).balance - prevEthBalance;
        console.log("Increased ETH balance: ", increasedEthBalance);
    }

    function test_CreateLiquidityPool() public {
        uint depositAmount = 10 ether;
        vm.deal(address(this), depositAmount);

        address tokenAddress = factory.createMemeToken("Test", "TEST", "img://img.png", "hello there");
        Token token = Token(tokenAddress);
        
        // Buy 20K tokens initially
        uint tokenQty = 80000 * 10 ** 18;
        uint requiredEth = factory.getRequiredEth(tokenAddress, tokenQty);
        
        factory.buyMemeToken{value: requiredEth}(tokenAddress, tokenQty);
        assertEq(token.balanceOf(address(this)), tokenQty);
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
        SimpleToken newToken = new SimpleToken("A", "A", initialMintValue);
        newToken.transfer(address(factory), initialMintValue);

        factory.withdrawTokens(address(newToken), initialMintValue);
        assertEq(newToken.balanceOf(address(this)), initialMintValue, "Withdraw Token failed");
    }
}
