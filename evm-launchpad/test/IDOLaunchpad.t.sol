// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/IDOLaunchpad.sol";
import { IERC20Metadata } from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

contract IDOLaunchpadTest is Test {
    IDOLaunchpad launchpad;
    IDOLaunchpad launchpadETH;
    IERC20Metadata paymentToken;
    IERC20Metadata memeCoinToken;
    address owner;
    address user;
    uint256 startTime;
    uint256 endTime;
    uint256 tokenPrice = 1 ether;
    uint256 minInvestment = 0.1 ether;
    uint256 maxInvestment = 10 ether;

    function setUp() public {
        owner = address(this);
        user = address(0x123);
        startTime = block.timestamp + 1 days;
        endTime = block.timestamp + 7 days;

        // Deploy mock ERC20 tokens
        paymentToken = IERC20Metadata(address(new ERC20Mock("PaymentToken", "PAY", owner, 0 ether)));
        memeCoinToken = IERC20Metadata(address(new ERC20Mock("MemeCoin", "MEME", owner, 0 ether)));

        // Deploy the IDOLaunchpad contract
        launchpad = new IDOLaunchpad(
            address(memeCoinToken),
            address(paymentToken),
            tokenPrice,
            minInvestment,
            maxInvestment,
            startTime,
            endTime
        );

        // Deploy the IDOLaunchpad contract without paymentToken
        launchpadETH = new IDOLaunchpad(
            address(memeCoinToken),
            address(0),
            tokenPrice,
            minInvestment,
            maxInvestment,
            startTime,
            endTime
        );

        // Mint tokens to the user for testing
        deal(address(paymentToken), user, 100 ether);
        // fund 10 ETH to user
        vm.deal(user, 10 ether);
        
        // Mint tokens to the launchpad for testing
        deal(address(memeCoinToken), address(launchpad), 10 ether);
        
    }   

    function testBuyTokensWithEthBeforeSaleStart() public {
        vm.expectRevert("Sale is not active");
        
        vm.startPrank(user); // Set user as msg.sender
        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice; //1 ETH
        launchpadETH.buyTokens{value: 1 ether}(amountToBuy);        
        vm.stopPrank();
    }

    function testBuyTokensWithEthAfterSaleEnd() public {
        vm.expectRevert("Sale is not active");
        
        vm.startPrank(user); 
        vm.warp(endTime + 1);
        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice;
        launchpadETH.buyTokens{value: 1 ether}(amountToBuy);        
        vm.stopPrank();
    }

    function testBuyTokensWithEthInsufficientETH() public {
        vm.expectRevert("Insufficient native coin sent");

        vm.startPrank(user);
        vm.warp(startTime + 1);
        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice;
        launchpadETH.buyTokens{value: 0.1 ether}(amountToBuy);
        vm.stopPrank();
    }

    function testBuyTokensWithEth() public {
        vm.startPrank(user);  
        vm.warp(startTime + 1); // for SaleInActive 
        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice; //1ETH
        launchpadETH.buyTokens{value: 1 ether}(amountToBuy);
        assertEq(launchpadETH.tokensPurchased(user), amountToBuy);
        assertEq(launchpadETH.investments(user), cost);
        assertEq(address(launchpadETH).balance, cost);
        vm.stopPrank();
    }

    function testBuyTokensWithERC20() public {
        vm.startPrank(user);
        vm.warp(startTime + 1);
        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice;

        paymentToken.approve(address(launchpad), cost);
        launchpad.buyTokens(amountToBuy);

        assertEq(launchpad.tokensPurchased(user), amountToBuy);
        assertEq(launchpad.investments(user), cost);
        assertEq(paymentToken.balanceOf(address(launchpad)), cost);

        vm.stopPrank();
    }

    function testClaimTokens() public {
        vm.warp(startTime + 1);
        vm.startPrank(user);

        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice;

        paymentToken.approve(address(launchpad), cost);
        launchpad.buyTokens(amountToBuy);

        vm.warp(endTime + 1);
        launchpad.claimTokens();

        assertEq(memeCoinToken.balanceOf(user), amountToBuy);
        assertEq(launchpad.tokensPurchased(user), amountToBuy);

        vm.stopPrank();
    }

    function testClaimTokensBeforeSaleEnd() public {
        vm.warp(startTime + 1);
        vm.startPrank(user);

        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice;

        paymentToken.approve(address(launchpad), cost);
        launchpad.buyTokens(amountToBuy);
        vm.warp(endTime - 1);

        vm.expectRevert("Sale has not ended yet");
        launchpad.claimTokens();
        vm.stopPrank();
    }

    function testWithdrawFunds() public {
        vm.startPrank(user);
        vm.warp(startTime + 1);
        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice;

        paymentToken.approve(address(launchpad), cost);
        launchpad.buyTokens(amountToBuy);

        vm.stopPrank();

        vm.warp(endTime + 1);
        launchpad.withdrawFunds(owner);

        assertEq(paymentToken.balanceOf(owner), cost);
        assertEq(paymentToken.balanceOf(address(launchpad)), 0);
    }

    function testWithdrawFundsByNotOwner() public {
        vm.startPrank(user);
        vm.warp(startTime + 1);
        uint256 amountToBuy = 1;
        uint256 cost = amountToBuy * tokenPrice;

        paymentToken.approve(address(launchpad), cost);
        launchpad.buyTokens(amountToBuy);
        vm.warp(endTime + 1);
        vm.expectRevert("Only the contract owner can call this function");

        launchpad.withdrawFunds(owner);
        vm.stopPrank();
    }

    function testWithdrawUnsoldTokens() public {
        vm.warp(endTime + 1);

        uint256 unsoldTokens = memeCoinToken.balanceOf(address(launchpad)) - launchpad.totalTokensSold();

        launchpad.withdrawUnsoldTokens();

        assertEq(memeCoinToken.balanceOf(owner), unsoldTokens);
    }
}

// Mock ERC20 token for testing purposes
contract ERC20Mock is IERC20Metadata {
    string public name;
    string public symbol;
    uint8 public decimals = 18;
    uint256 public totalSupply;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    constructor(
        string memory _name,
        string memory _symbol,
        address _initialAccount,
        uint256 _initialBalance
    ) {
        name = _name;
        symbol = _symbol;
        _mint(_initialAccount, _initialBalance);
    }

    function transfer(address recipient, uint256 amount) external returns (bool) {
        _transfer(msg.sender, recipient, amount);
        return true;
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transferFrom(address sender, address recipient, uint256 amount) external returns (bool) {
        allowance[sender][msg.sender] = allowance[sender][msg.sender] - amount;
        _transfer(sender, recipient, amount);
        return true;
    }

    function _transfer(address sender, address recipient, uint256 amount) internal {
        balanceOf[sender] = balanceOf[sender] - amount;
        balanceOf[recipient] = balanceOf[recipient] + amount;
        emit Transfer(sender, recipient, amount);
    }

    function _mint(address account, uint256 amount) internal {
        totalSupply = totalSupply + amount;
        balanceOf[account] = balanceOf[account] + amount;
        emit Transfer(address(0), account, amount);
    }
}
