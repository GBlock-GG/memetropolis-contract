// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "./Token.sol";
import "./library/LogExpMath.sol";
import "@uniswap-v2-core-1.0.1/contracts/interfaces/IUniswapV2Factory.sol";
import "@uniswap-v2-core-1.0.1/contracts/interfaces/IUniswapV2Pair.sol";
import "@uniswap-v2-periphery-1.1.0-beta.0/contracts/interfaces/IUniswapV2Router02.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {
    ILayerZeroEndpointV2,
    MessagingParams,
    MessagingReceipt,
    Origin,
    MessagingFee
} from "@layerzerolabs/lz-evm-protocol-v2/contracts/interfaces/ILayerZeroEndpointV2.sol";
import { OApp } from "@layerzerolabs/lz-evm-oapp-v2/contracts/oapp/OApp.sol";
import { OptionsBuilder } from "@layerzerolabs/lz-evm-oapp-v2/contracts/oapp/libs/OptionsBuilder.sol";
import {console} from "forge-std/console.sol";

/// @title Meme Token Factory
/// @notice This contract allows users to create and manage Meme tokens, which can be traded and sold on Uniswap.
/// @dev It includes features such as funding, token creation, Uniswap liquidity pool creation, and trading based on an exponential bonding curve.
contract TokenFactory is ReentrancyGuard, Ownable, OApp {
    using OptionsBuilder for bytes;

    /// @notice Data structure to hold information about each meme token.
    struct memeToken {
        string name;
        string symbol;
        string description;
        string tokenImageUrl;
        uint fundingRaised;
        bool isFundingFinished;
        address tokenAddress;
        address creatorAddress;
    }

    /// @notice Array to store the addresses of all created meme tokens.
    address[] public memeTokenAddresses;

    /// @notice Maps meme token address to its associated details in `memeToken` struct.
    mapping(address => memeToken) public addressToMemeTokenMapping;

    uint public immutable TOKEN_CREATOR_BONUS;
    uint public immutable PLATFORM_FEE;
    address public immutable PLATFORM_TREASURY_ADDRESS;

    address constant UNISWAP_V2_FACTORY_ADDRESS = 0x8909Dc15e40173Ff4699343b6eB8132c65e18eC6;
    address constant UNISWAP_V2_ROUTER_ADDRESS = 0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24;
    address constant LZ_ENDPOINT_V2_ADDRESS = 0x6EDCE65403992e310A62460808c4b910D972f10f; // 0x1a44076050125825900e736c501f859c50fE728c

    uint constant DECIMALS = 10 ** 18;
    uint public constant MAX_SUPPLY = 1000000 * DECIMALS;
    uint public constant INIT_SUPPLY = 20 * MAX_SUPPLY / 100;

    uint256 public constant INITIAL_PRICE = 2 * 10 ** 12;  // Initial price in wei (P0), 2 * 10^12
    uint256 public constant K = 5 * 10 ** 12;  // Growth rate (k), scaled to avoid precision loss (5 * 10^12)

    uint8 internal constant BUY_TYPE = 1;
    uint8 internal constant SELL_TYPE = 2;
    
    event CreatedMemeToken(address indexed tokenAddress, address indexed creator, string name, string symbol);
    event BoughtMemeToken(address indexed memeTokenAddress, address indexed user, uint tokenQty);
    event SoldMemeToken(address indexed memeTokenAddress, address indexed user, uint tokenQty);
    event WithdrawnETH(uint256 amount);
    event WithdrawnToken(address tokenAddress, uint256 amount);
    
    error TokenNotListed();
    error FundingAlreadyRaised();
    error IncorrectETHSent();
    error FailedToSendETH();
    error InsufficientBalance();
    error NotEnoughAvailableSupply();
    error SlippageExceeded();
    error ArrayLengthsMustMatch();

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param ethAmount The Eth amount.
    function buyCrosschainMemetoken(uint32 _dstEid, address memeTokenAddress, uint128 ethAmount) external payable {
        bytes memory message = abi.encode(BUY_TYPE, memeTokenAddress, msg.sender, ethAmount, 0);
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, ethAmount);
        _lzSend(
            _dstEid,
            message,
            options,
            MessagingFee(msg.value, 0),
            payable(msg.sender)
        );
    }

    /// @notice Use this function to estimate fees for your cross-chain buyCrosschainMemetoken()
    function quoteBuyCrossChainMemetoken(
        uint32 _dstEid,
        address memeTokenAddress,
        uint128 ethAmount
    ) public view returns (uint256 nativeFee, uint256 lzTokenFee) {
        bytes memory message = abi.encode(BUY_TYPE, memeTokenAddress, msg.sender, ethAmount, 0);
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, ethAmount);
        
        MessagingFee memory fee = _quote(_dstEid, message, options, false);
        return (fee.nativeFee, fee.lzTokenFee);
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The Token amount to sell.
    function sellCrosschainMemetoken(uint32 _dstEid, address memeTokenAddress, uint256 tokenQty) external payable {
        bytes memory message = abi.encode(SELL_TYPE, memeTokenAddress, msg.sender, 0, tokenQty);
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, 0);
        _lzSend(
            _dstEid,
            message,
            options,
            MessagingFee(msg.value, 0),
            payable(msg.sender)
        );
    }

    /// @notice Use this function to estimate fees for your cross-chain buyCrosschainMemetoken()
    function quoteSellCrossChainMemetoken(
        uint32 _dstEid,
        address memeTokenAddress,
        uint256 tokenQty
    ) public view returns (uint256 nativeFee, uint256 lzTokenFee) {
        bytes memory message = abi.encode(SELL_TYPE, memeTokenAddress, msg.sender, 0, tokenQty);
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, 0);
        
        MessagingFee memory fee = _quote(_dstEid, message, options, false);
        return (fee.nativeFee, fee.lzTokenFee);
    }

    function _lzReceive(
        Origin calldata,
        bytes32,
        bytes calldata payload,
        address,  // Executor address as specified by the OApp.
        bytes calldata  // Any extra data or options to trigger on receipt.
    ) internal override {
        // Decode the payload to get the message
        (uint8 msgType, address memeTokenAddress, address _to, uint256 ethAmount, uint256 tokenQty) = abi.decode(payload, (uint8, address, address, uint256, uint256));
        console.log("lzReceive msg.value: ", msg.value);
        console.log("lzReceive memeTokenAddress: ", memeTokenAddress);
        console.log("lzReceive ethAmount: ", ethAmount);
        
        if (msgType == BUY_TYPE) {
            if (msg.value < ethAmount)
                revert IncorrectETHSent();

            _buyMemeTokenInEth(memeTokenAddress, ethAmount, _to, 0);
        } else if (msgType == SELL_TYPE) {
            _sellMemeToken(memeTokenAddress, _to, tokenQty);
        }
    }

    constructor(
        address treasuryAddress,
        uint256 tokenCreatorBonus,
        uint256 platformFee,
        address _lzEndpoint
    )
        Ownable(msg.sender)
        OApp(_lzEndpoint, msg.sender)
    {
        PLATFORM_TREASURY_ADDRESS = treasuryAddress;
        TOKEN_CREATOR_BONUS = tokenCreatorBonus;
        PLATFORM_FEE = platformFee;
    }

    /// @notice Calculates the cost in wei for purchasing tokens using an exponential bonding curve.
    /// @param currentSupply The current token supply.
    /// @param tokensToBuy The number of tokens the user wants to buy.
    /// @return The total cost in wei required to purchase the tokens.
    function calculateCost(uint256 currentSupply, uint256 tokensToBuy) public pure returns (uint256) {
        
        // Calculate the exponent parts scaled to avoid precision loss
        uint256 exponent1 = (K * (currentSupply + tokensToBuy));
        uint256 exponent2 = (K * currentSupply);

        // Calculate e^(kx) using the exp function
        uint256 exp1 = LogExpMath.exp(exponent1);
        uint256 exp2 = LogExpMath.exp(exponent2);

        // Cost formula: (P0 / k) * (e^(k * (currentSupply + tokensToBuy)) - e^(k * currentSupply))
        // We use (P0 * 10^18) / k to keep the division safe from zero
        uint256 cost = (INITIAL_PRICE * (exp1 - exp2)) / K;  // Adjust for k scaling without dividing by zero
        return cost;
    }

    /// @notice Calculates the cost in wei for purchasing tokens using an exponential bonding curve.
    /// @param currentSupply The current token supply.
    /// @param ethAmount The ETH amount.
    /// @return The tokens to purchase.
    function calculateTokenAmount(uint256 currentSupply, uint256 ethAmount) public pure returns (uint256) {
        
        // Calculate e^(k * currentSupply)
        uint256 exp = LogExpMath.exp(K * currentSupply);
        console.log("exp: ", exp);

        // Calculate (ethAmount * K) / INIT_PRICE
        uint256 num = (ethAmount * K) / INITIAL_PRICE;
        console.log("num: ", num);

        // Calculate ln((ethAmount * K) / INIT_PRICE + e^(k * currentSupply))
        uint256 ln = LogExpMath.ln(num + exp);
        console.log("ln: ", ln);

        // formula: (ln((ethAmount * K) / INIT_PRICE + e^(k * currentSupply)) / K) - currentSupply
        uint256 tokenAmount = ln / K - currentSupply;  // Adjust for k scaling without dividing by zero
        console.log("tokenAmount: ", tokenAmount);
        return tokenAmount;
    }

    /// @notice Creates a new Meme Token.
    /// @param name The name of the meme token.
    /// @param symbol The symbol of the meme token.
    /// @param imageUrl The image URL for the meme token.
    /// @param description The description for the meme token.
    /// @return The address of the created meme token contract.
    function createMemeToken(string memory name, string memory symbol, string memory imageUrl, string memory description) external returns(address) {

        //should deploy the meme token, mint the initial supply to the token factory contract
        Token ct = new Token(name, symbol, MAX_SUPPLY, address(endpoint), msg.sender);
        address memeTokenAddress = address(ct);
        memeToken memory newlyCreatedToken = memeToken(name, symbol, description, imageUrl, 0, false, memeTokenAddress, msg.sender);
        memeTokenAddresses.push(memeTokenAddress);
        addressToMemeTokenMapping[memeTokenAddress] = newlyCreatedToken;

        emit CreatedMemeToken(memeTokenAddress, msg.sender, name, symbol);

        return memeTokenAddress;
    }

    /// @notice Retrieves all created Meme Tokens.
    /// @return An array of memeToken structs representing all created meme tokens.
    function getAllMemeTokens() external view returns(memeToken[] memory) {
        memeToken[] memory allTokens = new memeToken[](memeTokenAddresses.length);
        for (uint i = 0; i < memeTokenAddresses.length; i++) {
            allTokens[i] = addressToMemeTokenMapping[memeTokenAddresses[i]];
        }
        return allTokens;
    }

    /// @notice Calculates the required ETH for purchasing a certain quantity of meme tokens.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The token amount to buy.
    /// @return The required ETH in wei to purchase the given quantity of tokens.
    function getRequiredEth(address memeTokenAddress, uint tokenQty) external view returns (uint) {
        //check if memecoin is listed
        if (addressToMemeTokenMapping[memeTokenAddress].tokenAddress == address(0))
            revert TokenNotListed();

        Token memeTokenCt = Token(memeTokenAddress);
        uint currentSupplyScaled = (MAX_SUPPLY - memeTokenCt.balanceOf(address(this))) / DECIMALS;
        uint tokenQtyScaled = tokenQty / DECIMALS;
        uint requiredEth = calculateCost(currentSupplyScaled, tokenQtyScaled);

        return requiredEth;
    }

    /// @notice Calculates the required ETH for purchasing a certain quantity of meme tokens.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param ethAmount The ETH amount to purchase.
    /// @return The token out amount.
    function getTokenOutOnBuy(address memeTokenAddress, uint ethAmount) public view returns (uint) {
        //check if memecoin is listed
        require(addressToMemeTokenMapping[memeTokenAddress].tokenAddress!=address(0), "Token is not listed");

        Token memeTokenCt = Token(memeTokenAddress);
        uint currentSupplyScaled = (MAX_SUPPLY - memeTokenCt.balanceOf(address(this))) / DECIMALS;
        uint tokenAmountToPurchase = calculateTokenAmount(currentSupplyScaled, ethAmount) * DECIMALS;

        return tokenAmountToPurchase;
    }

    /// @notice Calculates the ETH to receive when sell certain quantity of meme tokens.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The token amount to sell.
    /// @return The ETH in wei to receive for quantity of tokens sell.
    function getEthAmountOnSell(address memeTokenAddress, uint tokenQty) public view returns (uint) {
        //check if memecoin is listed
        require(addressToMemeTokenMapping[memeTokenAddress].tokenAddress!=address(0), "Token is not listed");

        Token memeTokenCt = Token(memeTokenAddress);
        uint currentSupplyScaled = (MAX_SUPPLY - memeTokenCt.balanceOf(address(this))) / DECIMALS;
        uint tokenQtyScaled = tokenQty / DECIMALS;
        uint requiredEth = calculateCost(currentSupplyScaled - tokenQtyScaled, tokenQtyScaled);

        return requiredEth;
    }

    /// @notice Calculate current token price.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @return The price in ETH with 18 decimals.
    function getCurrentTokenPrice(address memeTokenAddress) public view returns (uint) {
        //check if memecoin is listed
        if (addressToMemeTokenMapping[memeTokenAddress].tokenAddress == address(0))
            revert TokenNotListed();

        Token memeTokenCt = Token(memeTokenAddress);
        uint tokenBalance = memeTokenCt.balanceOf(address(this));
        if (tokenBalance == 0)
            tokenBalance = INIT_SUPPLY;
        uint currentSupplyScaled = (MAX_SUPPLY - tokenBalance) / DECIMALS;

        uint256 exponent = K * currentSupplyScaled;
        uint256 price = INITIAL_PRICE * LogExpMath.exp(exponent) / DECIMALS;

        return price;
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The token amount to buy.
    function buyMemeToken(address memeTokenAddress, uint tokenQty) external payable {

        //check if memecoin is listed
        if (addressToMemeTokenMapping[memeTokenAddress].tokenAddress == address(0))
            revert TokenNotListed();
        
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];


        Token memeTokenCt = Token(memeTokenAddress);

        // check to ensure funding goal is not met
        if (listedToken.isFundingFinished)
            revert FundingAlreadyRaised();

        // check to ensure there is enough supply to facilitate the purchase
        uint available_qty = memeTokenCt.balanceOf(address(this)) - INIT_SUPPLY;

        if (tokenQty > available_qty)
            revert NotEnoughAvailableSupply();

        // calculate the cost for purchasing tokenQty tokens as per the exponential bonding curve formula
        uint currentSupplyScaled = (MAX_SUPPLY - memeTokenCt.balanceOf(address(this))) / DECIMALS;
        uint tokenQtyScaled = tokenQty / DECIMALS;
        uint requiredEth = calculateCost(currentSupplyScaled, tokenQtyScaled);

        // check if user has sent correct value of eth to facilitate this purchase
        if (msg.value < requiredEth)
            revert IncorrectETHSent();

        // Increment the funding
        listedToken.fundingRaised += msg.value;

        if(available_qty <= tokenQty) {
            // create liquidity pool and send fees
            _onBondingCurveFinish(memeTokenAddress);
        }

        // mint the tokens
        memeTokenCt.transfer(msg.sender, tokenQty);

        // refund rest funds to user
        if (msg.value > requiredEth) {
            uint refundAmount = msg.value - requiredEth;
            (bool success, ) = payable(msg.sender).call{value: refundAmount}("");
            if (!success)
                revert FailedToSendETH();
        }

        emit BoughtMemeToken(memeTokenAddress, msg.sender, tokenQty);
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQtyMin The minimum token amount to buy.
    function buyMemeTokenInEth(address memeTokenAddress, uint tokenQtyMin) external payable {
        _buyMemeTokenInEth(memeTokenAddress, msg.value, msg.sender, tokenQtyMin);
    }

    /// @notice Internal function which allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param ethAmount The ETH amount to send.
    /// @param _to The address to receive tokens.
    /// @param tokenQtyMin The minimum token amount to buy.
    function _buyMemeTokenInEth(address memeTokenAddress, uint ethAmount, address _to, uint tokenQtyMin) internal {
        //check if memecoin is listed
        if (addressToMemeTokenMapping[memeTokenAddress].tokenAddress == address(0))
            revert TokenNotListed();
        
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];


        Token memeTokenCt = Token(memeTokenAddress);

        // check to ensure funding goal is not met
        if (listedToken.isFundingFinished)
            revert FundingAlreadyRaised();

        uint currentSupplyScaled = (MAX_SUPPLY - memeTokenCt.balanceOf(address(this))) / DECIMALS;
        console.log("lzReceive currentSupplyScaled: ", currentSupplyScaled);
        uint tokenAmountToPurchase = calculateTokenAmount(currentSupplyScaled, ethAmount) * DECIMALS;
        console.log("lzReceive tokenAmountToPurchase: ", tokenAmountToPurchase);
        if (tokenAmountToPurchase < tokenQtyMin)
            revert SlippageExceeded();

        // check to ensure there is enough supply to facilitate the purchase
        uint available_qty = memeTokenCt.balanceOf(address(this)) - INIT_SUPPLY;
        console.log("lzReceive available_qty: ", available_qty);

        if (tokenAmountToPurchase > available_qty)
            revert NotEnoughAvailableSupply();

        // Increment the funding
        listedToken.fundingRaised += ethAmount;
        
        if(available_qty <= tokenAmountToPurchase) {
            // create liquidity pool and send fees
            _onBondingCurveFinish(memeTokenAddress);
        }

        // mint the tokens
        memeTokenCt.transfer(_to, tokenAmountToPurchase);

        emit BoughtMemeToken(memeTokenAddress, _to, tokenAmountToPurchase);
    }

    /// @notice Allows users to sell meme tokens for ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The number of tokens to sell.
    /// @return The amount of ETH received in return.
    function sellMemeToken(address memeTokenAddress, uint tokenQty) external nonReentrant returns(uint) {
        return _sellMemeToken(memeTokenAddress, msg.sender, tokenQty);
    }

    /// @notice Internal function which allows users to sell meme tokens for ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The number of tokens to sell.
    /// @return The amount of ETH received in return.
    function _sellMemeToken(address memeTokenAddress, address from, uint tokenQty) internal returns(uint) {
        //check if memecoin is listed
        if (addressToMemeTokenMapping[memeTokenAddress].tokenAddress == address(0))
            revert TokenNotListed();

        Token memeTokenCt = Token(memeTokenAddress);
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];

        // check to ensure funding goal is not met
        if (listedToken.isFundingFinished)
            revert FundingAlreadyRaised();

        memeTokenCt.transferFrom(from, address(this), tokenQty);

        // ethAmount to send = P0 * (e^(k*c)- e^(k*(c-x))) / k
        uint currentSupplyScaled = (MAX_SUPPLY - memeTokenCt.balanceOf(address(this))) / DECIMALS;
        uint tokenQtyScaled = tokenQty / DECIMALS;
        uint ethAmount = calculateCost(currentSupplyScaled - tokenQtyScaled, tokenQtyScaled);

        // decrease funding raised amount
        listedToken.fundingRaised -= ethAmount;

        (bool success, ) = payable(from).call{value: ethAmount}("");
        if (!success)
            revert FailedToSendETH();

        emit SoldMemeToken(memeTokenAddress, from, tokenQty);

        return ethAmount;
    }

    /// @notice Internal function to Add liquidity on Uniswap V2 and send fees.
    /// @param memeTokenAddress The address of the meme token contract.
    function _onBondingCurveFinish(address memeTokenAddress) internal {
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];

        // create liquidity pool
        address pool = _createLiquidityPool(memeTokenAddress);
    
        // provide liquidity
        uint tokenAmount = INIT_SUPPLY;
        uint ethAmount = listedToken.fundingRaised - TOKEN_CREATOR_BONUS - PLATFORM_FEE;
        uint liquidity = _provideLiquidity(memeTokenAddress, tokenAmount, ethAmount);
    
        // burn lp token
        _burnLpTokens(pool, liquidity);

        listedToken.isFundingFinished = true;

        // transfer fee to token creator
        (bool success, ) = payable(listedToken.creatorAddress).call{value: TOKEN_CREATOR_BONUS}("");
        if (!success)
            revert FailedToSendETH();

        // transfer fee to platform
        (bool success1, ) = payable(PLATFORM_TREASURY_ADDRESS).call{value: PLATFORM_FEE}("");
        if (!success1)
            revert FailedToSendETH();
    }

    /// @notice Internal function to create a liquidity pool on Uniswap V2 for the meme token.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @return The address of the created Uniswap V2 pair.
    function _createLiquidityPool(address memeTokenAddress) internal returns(address) {
        IUniswapV2Factory factory = IUniswapV2Factory(UNISWAP_V2_FACTORY_ADDRESS);
        IUniswapV2Router02 router = IUniswapV2Router02(UNISWAP_V2_ROUTER_ADDRESS);
        address pair = factory.createPair(memeTokenAddress, router.WETH());
        return pair;
    }

    /// @notice Internal function to provide liquidity to the Uniswap V2 pool.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenAmount The amount of meme tokens.
    /// @param ethAmount The amount of ETH.
    /// @return The amount of liquidity provided.
    function _provideLiquidity(address memeTokenAddress, uint tokenAmount, uint ethAmount) internal returns(uint){
        IERC20(memeTokenAddress).approve(UNISWAP_V2_ROUTER_ADDRESS, tokenAmount);
        IUniswapV2Router02 router = IUniswapV2Router02(UNISWAP_V2_ROUTER_ADDRESS);
        (, , uint liquidity) = router.addLiquidityETH{
            value: ethAmount
        }(memeTokenAddress, tokenAmount, tokenAmount, ethAmount, address(this), block.timestamp);
        return liquidity;
    }

    /// @notice Internal function to burn the liquidity pool tokens.
    /// @param pool The address of the liquidity pool.
    /// @param liquidity The amount of liquidity to burn.
    function _burnLpTokens(address pool, uint liquidity) internal {
        IUniswapV2Pair uniswapv2pairct = IUniswapV2Pair(pool);
        uniswapv2pairct.transfer(address(0), liquidity);
    }

    /// @notice Allows the owner to withdraw ETH from the contract
    /// @param amount The amount of ETH to withdraw
    /// @dev Emits a WithdrawnETH event on successful withdrawal
    function withdrawETH(uint256 amount) external onlyOwner {
        if (address(this).balance < amount)
            revert InsufficientBalance();
        
        (bool success, ) = payable(owner()).call{value: amount}("");
        if (!success)
            revert InsufficientBalance();
        emit WithdrawnETH(amount);
    }

    /// @notice Allows the owner to withdraw ERC20 token from the contract
    /// @param amount The amount of token to withdraw
    /// @dev Emits a WithdrawnToken event on successful withdrawal
    function withdrawTokens(address tokenAddress, uint256 amount) external onlyOwner {
        if (IERC20(tokenAddress).balanceOf(address(this)) < amount)
            revert InsufficientBalance();
        IERC20(tokenAddress).transfer(owner(), amount);

        emit WithdrawnToken(tokenAddress, amount);
    }

    /// @notice Fallback function to allow the contract to receive ETH.
    receive() external payable {}
}
