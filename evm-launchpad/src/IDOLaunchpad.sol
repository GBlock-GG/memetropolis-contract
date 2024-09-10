// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { IERC20Metadata } from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { SafeMath } from "@openzeppelin/contracts/utils/math/SafeMath.sol";

contract IDOLaunchpad {
    using SafeMath for uint256;
    using SafeERC20 for IERC20Metadata;

    address public owner;
    uint256 public tokenPrice;
    uint256 public minInvestment;
    uint256 public maxInvestment;
    uint256 public totalTokensSold;
    uint256 public claimedAmount;
    uint256 public startTime;
    uint256 public endTime;

    IERC20Metadata public memeCoinToken;
    address public paymentToken; // Use address instead of IERC20Metadata for flexibility

    mapping(address => uint256) public investments;
    mapping(address => uint256) public tokensPurchased;
    mapping(address => bool) public hasClaimedTokens;

    event TokensPurchased(address indexed buyer, uint256 amount, uint256 cost);
    event TokensClaimed(address indexed buyer, uint256 amount);
    event FeesWithdrawn(address indexed beneficiary, uint256 amount);

    modifier onlyOwner() {
        require(msg.sender == owner, "Only the contract owner can call this function");
        _;
    }

    modifier saleIsActive() {
        require(block.timestamp >= startTime && block.timestamp <= endTime, "Sale is not active");
        _;
    }

    modifier saleHasEnded() {
        require(block.timestamp > endTime, "Sale has not ended yet");
        _;
    }

    constructor(
        address _tokenAddress,
        address _paymentTokenAddress,
        uint256 _price,
        uint256 _minInvestment,
        uint256 _maxInvestment,
        uint256 _startTime,
        uint256 _endTime
    ) {
        require(_endTime > _startTime, "End time must be after start time");

        owner = msg.sender;
        memeCoinToken = IERC20Metadata(_tokenAddress);
        paymentToken = _paymentTokenAddress;
        tokenPrice = _price;
        minInvestment = _minInvestment;
        maxInvestment = _maxInvestment;
        startTime = _startTime;
        endTime = _endTime;
    }

    function buyTokens(uint256 _amount) external payable saleIsActive {
        uint256 cost = _amount.mul(tokenPrice);
        require(cost >= minInvestment && cost <= maxInvestment, "Investment out of range");

        if (paymentToken == address(0)) {
            // Native token payment
            require(msg.value >= cost, "Insufficient native coin sent");

            investments[msg.sender] = investments[msg.sender].add(msg.value);
            tokensPurchased[msg.sender] = tokensPurchased[msg.sender].add(_amount);
            totalTokensSold = totalTokensSold.add(_amount);

            emit TokensPurchased(msg.sender, _amount, cost);
        } else {
            // ERC20 token payment
            IERC20Metadata paymentERC20 = IERC20Metadata(paymentToken);
            require(paymentERC20.balanceOf(msg.sender) >= cost, "Insufficient payment token balance");

            paymentERC20.safeTransferFrom(msg.sender, address(this), cost);

            investments[msg.sender] = investments[msg.sender].add(cost);
            tokensPurchased[msg.sender] = tokensPurchased[msg.sender].add(_amount);
            totalTokensSold = totalTokensSold.add(_amount);

            emit TokensPurchased(msg.sender, _amount, cost);
        }
    }

    function claimTokens() external saleHasEnded {
        require(!hasClaimedTokens[msg.sender], "Tokens already claimed");
        uint256 amount = tokensPurchased[msg.sender];
        require(amount > 0, "No tokens to claim");

        hasClaimedTokens[msg.sender] = true;
        memeCoinToken.safeTransfer(msg.sender, amount);
        claimedAmount = claimedAmount.add(amount);

        emit TokensClaimed(msg.sender, amount);
    }

    function withdrawFunds(address _beneficiary) external onlyOwner saleHasEnded {
        if (paymentToken == address(0)) {
            // Withdraw native coins
            uint256 balance = address(this).balance;
            require(balance > 0, "No funds to withdraw");

            payable(_beneficiary).transfer(balance);

            emit FeesWithdrawn(_beneficiary, balance);
        } else {
            // Withdraw ERC20 tokens
            IERC20Metadata paymentERC20 = IERC20Metadata(paymentToken);
            uint256 balance = paymentERC20.balanceOf(address(this));
            require(balance > 0, "No funds to withdraw");

            paymentERC20.safeTransfer(_beneficiary, balance);

            emit FeesWithdrawn(_beneficiary, balance);
        }
    }

    function withdrawUnsoldTokens() external onlyOwner saleHasEnded {
        uint256 unsoldTokens = memeCoinToken.balanceOf(address(this)).sub(totalTokensSold.sub(claimedAmount));
        memeCoinToken.safeTransfer(owner, unsoldTokens);
    }
}
