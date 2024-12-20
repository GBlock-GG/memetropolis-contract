// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "./Token.sol";
import "./library/LogExpMath.sol";
import "./interface/IUniswapV2Factory.sol";
import "./interface/IUniswapV2Pair.sol";
import "./interface/IUniswapV2Router02.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {
    Origin,
    MessagingFee
} from "@layerzerolabs/lz-evm-protocol-v2/contracts/interfaces/ILayerZeroEndpointV2.sol";
import { OApp } from "@layerzerolabs/lz-evm-oapp-v2/contracts/oapp/OApp.sol";
import { OptionsBuilder } from "@layerzerolabs/lz-evm-oapp-v2/contracts/oapp/libs/OptionsBuilder.sol";

/// @title Meme Token Factory
/// @notice This contract allows users to create and manage Meme tokens, which can be traded and sold on Uniswap.
/// @dev It includes features such as funding, token creation, Uniswap liquidity pool creation, and trading based on an exponential bonding curve.
contract TokenFactory is ReentrancyGuard, Ownable, OApp {
    using OptionsBuilder for bytes;

    /// @notice Data structure to hold custom token informations.
    struct AdvancedInfo {
        uint k;
        uint initialPrice;
        uint maxSupply;
        uint salesRatio;
        uint reservedRatio;
        uint liquidityPoolRatio;
        uint launchDate;
        uint maximumPerUser;
    }

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
        AdvancedInfo advancedInfo;
    }

    /// @notice Maps meme token address to its associated details in `memeToken` struct.
    mapping(address => memeToken) public addressToMemeTokenMapping;

    mapping(address => mapping(address => uint)) private userBoughtAmount;

    uint256 private immutable TOKEN_CREATOR_BONUS;
    uint256 private immutable PLATFORM_FEE;
    address private immutable PLATFORM_TREASURY_ADDRESS;

    address private immutable UNISWAP_V2_FACTORY_ADDRESS;
    address private immutable UNISWAP_V2_ROUTER_ADDRESS;
    
    uint256 private constant DECIMALS = 10 ** 18;
    
    uint256 private constant DEFAULT_MAX_SUPPLY = 1000000 * DECIMALS;

    uint256 private constant DEFAULT_SALES_RATIO = 8000;
    uint256 private constant DEFAULT_RESERVED_RATIO = 0;
    uint256 private constant DEFAULT_LIQUIDITY_RATIO = 2000;
    
    uint256 private immutable DEFAULT_INITIAL_PRICE;  // Initial price in wei (P0), 2 * 10^12
    uint256 private constant DEFAULT_K = 5 * 10 ** 12;  // Growth rate (k), scaled to avoid precision loss (5 * 10^12)

    uint8 private constant BUY_TYPE = 1;
    uint8 private constant SELL_TYPE = 2;
    
    event CreatedMemeToken(address indexed tokenAddress, address indexed creator, string name, string symbol);
    event BoughtMemeToken(address indexed memeTokenAddress, address indexed user, uint tokenQty, uint ethAmount);
    event BoughtCrosschainMemeToken(uint32 indexed srcEid, address indexed memeTokenAddress, address indexed user, uint tokenQty, uint ethAmount);
    event SoldMemeToken(address indexed memeTokenAddress, address indexed user, uint tokenQty, uint ethAmount);
    event SoldCrosschainMemeToken(uint32 indexed srcEid, address indexed memeTokenAddress, address indexed user, uint tokenQty, uint ethAmount);
    
    error TokenNotListed();
    error TokenNotLaunched();
    error FundingAlreadyRaised();
    error IncorrectETHSent();
    error FailedToSendETH();
    error NotEnoughAvailableSupply();
    error SlippageExceeded();
    error IncorrectAddress();
    error InvalidTokenCreation();
    error UserBuyLimitExceeded();

    constructor(
        address treasuryAddress,
        uint256 tokenCreatorBonus,
        uint256 platformFee,
        address _lzEndpoint,
        uint256 initialPrice,
        address _uniswapV2Factory,
        address _uniswapV2Router
    )
        Ownable(msg.sender)
        OApp(_lzEndpoint, msg.sender)
    {
        if (treasuryAddress == address(0) || _lzEndpoint == address(0))
            revert IncorrectAddress();

        PLATFORM_TREASURY_ADDRESS = treasuryAddress;
        TOKEN_CREATOR_BONUS = tokenCreatorBonus;
        PLATFORM_FEE = platformFee;
        DEFAULT_INITIAL_PRICE = initialPrice;
        UNISWAP_V2_FACTORY_ADDRESS = _uniswapV2Factory;
        UNISWAP_V2_ROUTER_ADDRESS = _uniswapV2Router;
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param recipientAddress The recipient address.
    /// @param ethAmount The Eth amount.
    function buyCrosschainMemetoken(uint32 _dstEid, bytes32 memeTokenAddress, bytes32 recipientAddress, uint128 ethAmount) external payable {
        bytes memory message = abi.encodePacked(BUY_TYPE, memeTokenAddress, recipientAddress, ethAmount, uint256(0));
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, ethAmount);
        _lzSend(
            _dstEid,
            message,
            options,
            MessagingFee(msg.value, 0),
            msg.sender
        );
    }

    /// @notice Use this function to estimate fees for your cross-chain buyCrosschainMemetoken()
    function quoteBuyCrossChainMemetoken(
        uint32 _dstEid,
        bytes32 memeTokenAddress,
        bytes32 recipientAddress,
        uint128 ethAmount
    ) external view returns (uint256 nativeFee, uint256 lzTokenFee) {
        bytes memory message = abi.encodePacked(BUY_TYPE, memeTokenAddress, recipientAddress, ethAmount, uint256(0));
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, ethAmount);
        MessagingFee memory fee = _quote(_dstEid, message, options, false);
        return (fee.nativeFee, fee.lzTokenFee);
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param recipientAddress The recipient address.
    /// @param tokenQty The Token amount to sell.
    function sellCrosschainMemetoken(uint32 _dstEid, bytes32 memeTokenAddress, bytes32 recipientAddress, uint256 tokenQty) external payable {
        bytes memory message = abi.encodePacked(SELL_TYPE, memeTokenAddress, recipientAddress, uint128(0), tokenQty);
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, 0);
        _lzSend(
            _dstEid,
            message,
            options,
            MessagingFee(msg.value, 0),
            msg.sender
        );
    }

    /// @notice Use this function to estimate fees for your cross-chain buyCrosschainMemetoken()
    function quoteSellCrossChainMemetoken(
        uint32 _dstEid,
        bytes32 memeTokenAddress,
        bytes32 recipientAddress,
        uint256 tokenQty
    ) external view returns (uint256 nativeFee, uint256 lzTokenFee) {
        bytes memory message = abi.encodePacked(SELL_TYPE, memeTokenAddress, recipientAddress, uint128(0), tokenQty);
        bytes memory options = OptionsBuilder.newOptions().addExecutorLzReceiveOption(200000, 0);
        
        MessagingFee memory fee = _quote(_dstEid, message, options, false);
        return (fee.nativeFee, fee.lzTokenFee);
    }

    function _lzReceive(
        Origin calldata origin,
        bytes32,
        bytes calldata payload,
        address,  // Executor address as specified by the OApp.
        bytes calldata  // Any extra data or options to trigger on receipt.
    ) internal override {
        // Decode the payload to get the message
        (uint8 msgType, bytes32 memeTokenAddressBytes, bytes32 _toBytes, uint256 ethAmount, uint256 tokenQty) = decodeMessage(payload);
        address memeTokenAddress = address(uint160(uint256(memeTokenAddressBytes)));
        address _to = address(uint160(uint256(_toBytes)));

        if (msgType == BUY_TYPE) {
            if (msg.value < ethAmount)
                revert IncorrectETHSent();

            uint256 tokenAmount = _buyMemeTokenInEth(memeTokenAddress, ethAmount, _to, 0);
            emit BoughtCrosschainMemeToken(origin.srcEid, memeTokenAddress, _to, tokenAmount, ethAmount);
        } else if (msgType == SELL_TYPE) {
            uint256 ethAmountToReceive =  _sellMemeToken(memeTokenAddress, _to, tokenQty);
            emit SoldCrosschainMemeToken(origin.srcEid, memeTokenAddress, _to, tokenQty, ethAmountToReceive);
        }
    }

    function decodeMessage(bytes memory message) private pure returns (uint8 msgType, bytes32 memeTokenAddressBytes, bytes32 recipientAddressBytes, uint256 ethAmount, uint256 tokenQty) {
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
    function calculateCost(uint256 currentSupply, uint256 tokensToBuy, uint256 tokenK, uint256 tokenInitialPrice) private pure returns (uint256) {
        
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
    function calculateTokenAmount(uint256 currentSupply, uint256 ethAmount, uint256 tokenK, uint256 tokenInitialPrice) private pure returns (uint256) {
        
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

    /// @notice Creates a new Meme Token.
    /// @return The address of the created meme token contract.
    function createMemeToken(
        string memory name,
        string memory symbol,
        string memory imageUrl,
        string memory description,
        uint k,
        uint initialPrice,
        uint maxSupply,
        uint salesRatio,
        uint reservedRatio,
        uint liquidityPoolRatio,
        uint launchDate,
        uint maximumPerUser
    ) external returns(address) {
        if (bytes(name).length == 0 || bytes(symbol).length == 0)
            revert InvalidTokenCreation();

        //should deploy the meme token, mint the initial supply to the token factory contract
        uint realMaxSupply = maxSupply == 0 ? DEFAULT_MAX_SUPPLY : maxSupply;
        Token ct = new Token(name, symbol, realMaxSupply, address(endpoint), msg.sender);
        if (reservedRatio > 0) {
            uint reservedSupply = reservedRatio * realMaxSupply / 10000;
            ct.transfer(msg.sender, reservedSupply);
        }
        address memeTokenAddress = address(ct);
        memeToken memory newlyCreatedToken = memeToken(
            name,
            symbol,
            description,
            imageUrl,
            0,
            false,
            memeTokenAddress,
            msg.sender,
            AdvancedInfo(
                k == 0 ? DEFAULT_K : k,
                initialPrice == 0 ? DEFAULT_INITIAL_PRICE : initialPrice,
                realMaxSupply,
                salesRatio == 0 ? DEFAULT_SALES_RATIO : salesRatio,
                reservedRatio == 0 ? DEFAULT_RESERVED_RATIO : reservedRatio,
                liquidityPoolRatio == 0 ? DEFAULT_LIQUIDITY_RATIO : liquidityPoolRatio,
                launchDate,
                maximumPerUser
            )
        );
        addressToMemeTokenMapping[memeTokenAddress] = newlyCreatedToken;

        emit CreatedMemeToken(memeTokenAddress, msg.sender, name, symbol);

        return memeTokenAddress;
    }

    /// @notice Calculates the required ETH for purchasing a certain quantity of meme tokens.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The token amount to buy.
    /// @return The required ETH in wei to purchase the given quantity of tokens.
    function getRequiredEth(address memeTokenAddress, uint tokenQty) external view returns (uint) {
        memeToken storage token = addressToMemeTokenMapping[memeTokenAddress];
        
        uint tokenTotalSupply = (10000 - token.advancedInfo.reservedRatio) * token.advancedInfo.maxSupply / 10000;
        uint currentSupply = tokenTotalSupply - IERC20(memeTokenAddress).balanceOf(address(this));
        uint requiredEth = calculateCost(currentSupply, tokenQty, token.advancedInfo.k, token.advancedInfo.initialPrice);

        return requiredEth;
    }

    /// @notice Calculates the required ETH for purchasing a certain quantity of meme tokens.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param ethAmount The ETH amount to purchase.
    /// @return The token out amount.
    function getTokenOutOnBuy(address memeTokenAddress, uint ethAmount) external view returns (uint) {
        memeToken storage token = addressToMemeTokenMapping[memeTokenAddress];

        uint tokenTotalSupply = (10000 - token.advancedInfo.reservedRatio) * token.advancedInfo.maxSupply / 10000;
        uint currentSupply = tokenTotalSupply - IERC20(memeTokenAddress).balanceOf(address(this));
        uint tokenAmountToPurchase = calculateTokenAmount(currentSupply, ethAmount, token.advancedInfo.k, token.advancedInfo.initialPrice);

        return tokenAmountToPurchase;
    }

    /// @notice Calculates the ETH to receive when sell certain quantity of meme tokens.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The token amount to sell.
    /// @return The ETH in wei to receive for quantity of tokens sell.
    function getEthAmountOnSell(address memeTokenAddress, uint tokenQty) external view returns (uint) {
        memeToken storage token = addressToMemeTokenMapping[memeTokenAddress];

        uint tokenTotalSupply = (10000 - token.advancedInfo.reservedRatio) * token.advancedInfo.maxSupply / 10000;
        uint currentSupply = tokenTotalSupply - IERC20(memeTokenAddress).balanceOf(address(this));
        uint requiredEth = calculateCost(currentSupply - tokenQty, tokenQty, token.advancedInfo.k, token.advancedInfo.initialPrice);

        return requiredEth;
    }

    /// @notice Calculate current token price.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @return The price in ETH with 18 decimals.
    function getCurrentTokenPrice(address memeTokenAddress) external view returns (uint) {
        memeToken storage token = addressToMemeTokenMapping[memeTokenAddress];

        uint tokenBalance = IERC20(memeTokenAddress).balanceOf(address(this));
        if (tokenBalance == 0)
            tokenBalance = token.advancedInfo.liquidityPoolRatio * token.advancedInfo.maxSupply / 10000;
        uint tokenTotalSupply = (10000 - token.advancedInfo.reservedRatio) * token.advancedInfo.maxSupply / 10000;
        
        uint256 exponent = token.advancedInfo.k * (tokenTotalSupply - tokenBalance) / DECIMALS;
        uint256 price = token.advancedInfo.initialPrice * LogExpMath.exp(exponent) / DECIMALS;

        return price;
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The token amount to buy.
    function buyMemeToken(address memeTokenAddress, uint tokenQty) external payable {
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];
        //check if memecoin is listed
        if (listedToken.tokenAddress == address(0))
            revert TokenNotListed();
        
        //check if launch date of memecoin is arrived
        if (listedToken.advancedInfo.launchDate > block.timestamp)
            revert TokenNotLaunched();

        IERC20 memeTokenCt = IERC20(memeTokenAddress);

        // check to ensure funding goal is not met
        if (listedToken.isFundingFinished)
            revert FundingAlreadyRaised();

        // check to ensure there is enough supply to facilitate the purchase
        uint initSupply = listedToken.advancedInfo.liquidityPoolRatio * listedToken.advancedInfo.maxSupply / 10000;
        uint available_qty = memeTokenCt.balanceOf(address(this)) - initSupply;

        if (tokenQty > available_qty)
            revert NotEnoughAvailableSupply();

        // calculate the cost for purchasing tokenQty tokens as per the exponential bonding curve formula
        uint tokenTotalSupply = (10000 - listedToken.advancedInfo.reservedRatio) * listedToken.advancedInfo.maxSupply / 10000;
        uint currentSupply = tokenTotalSupply - memeTokenCt.balanceOf(address(this));
        uint requiredEth = calculateCost(currentSupply, tokenQty, listedToken.advancedInfo.k, listedToken.advancedInfo.initialPrice);
        if (listedToken.advancedInfo.maximumPerUser > 0 && userBoughtAmount[memeTokenAddress][msg.sender] + requiredEth > listedToken.advancedInfo.maximumPerUser)
            revert UserBuyLimitExceeded();

        // check if user has sent correct value of eth to facilitate this purchase
        if (msg.value < requiredEth)
            revert IncorrectETHSent();

        // Increment the funding
        listedToken.fundingRaised += requiredEth;

        if(available_qty == tokenQty) {
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

        userBoughtAmount[memeTokenAddress][msg.sender] += requiredEth;

        emit BoughtMemeToken(memeTokenAddress, msg.sender, tokenQty, requiredEth);
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQtyMin The minimum token amount to buy.
    function buyMemeTokenInEth(address memeTokenAddress, uint tokenQtyMin) external payable {
        uint256 tokenAmount = _buyMemeTokenInEth(memeTokenAddress, msg.value, msg.sender, tokenQtyMin);

        emit BoughtMemeToken(memeTokenAddress, msg.sender, tokenAmount, msg.value);
    }

    /// @notice Private function which allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param ethAmount The ETH amount to send.
    /// @param _to The address to receive tokens.
    /// @param tokenQtyMin The minimum token amount to buy.
    function _buyMemeTokenInEth(address memeTokenAddress, uint ethAmount, address _to, uint tokenQtyMin) private returns (uint256) {
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];
        //check if memecoin is listed
        if (listedToken.tokenAddress == address(0))
            revert TokenNotListed();
        
        //check if launch date of memecoin is arrived
        if (listedToken.advancedInfo.launchDate > block.timestamp)
            revert TokenNotLaunched();

        if (listedToken.advancedInfo.maximumPerUser > 0 && userBoughtAmount[memeTokenAddress][_to] + ethAmount > listedToken.advancedInfo.maximumPerUser)
            revert UserBuyLimitExceeded();

        Token memeTokenCt = Token(memeTokenAddress);

        // check to ensure funding goal is not met
        if (listedToken.isFundingFinished)
            revert FundingAlreadyRaised();

        uint tokenTotalSupply = (10000 - listedToken.advancedInfo.reservedRatio) * listedToken.advancedInfo.maxSupply / 10000;
        uint currentSupply = tokenTotalSupply - memeTokenCt.balanceOf(address(this));
        uint tokenAmountToPurchase = calculateTokenAmount(currentSupply, ethAmount, listedToken.advancedInfo.k, listedToken.advancedInfo.initialPrice);
        if (tokenAmountToPurchase < tokenQtyMin)
            revert SlippageExceeded();

        // check to ensure there is enough supply to facilitate the purchase
        uint initSupply = listedToken.advancedInfo.liquidityPoolRatio * listedToken.advancedInfo.maxSupply / 10000;
        uint available_qty = memeTokenCt.balanceOf(address(this)) - initSupply;
        
        if (tokenAmountToPurchase > available_qty)
            revert NotEnoughAvailableSupply();

        // Increment the funding
        listedToken.fundingRaised += ethAmount;
        
        if(available_qty == tokenAmountToPurchase) {
            // create liquidity pool and send fees
            _onBondingCurveFinish(memeTokenAddress);
        }

        // transfer tokens
        memeTokenCt.transfer(_to, tokenAmountToPurchase);
        userBoughtAmount[memeTokenAddress][_to] += ethAmount;
        return tokenAmountToPurchase;
    }

    /// @notice Allows users to sell meme tokens for ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The number of tokens to sell.
    /// @return The amount of ETH received in return.
    function sellMemeToken(address memeTokenAddress, uint tokenQty) external nonReentrant returns(uint) {
        uint256 ethAmount = _sellMemeToken(memeTokenAddress, msg.sender, tokenQty);

        emit SoldMemeToken(memeTokenAddress, msg.sender, tokenQty, ethAmount);
        return ethAmount;
    }

    /// @notice Private function which allows users to sell meme tokens for ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The number of tokens to sell.
    /// @return The amount of ETH received in return.
    function _sellMemeToken(address memeTokenAddress, address from, uint tokenQty) private returns(uint) {
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];
        //check if memecoin is listed
        if (listedToken.tokenAddress == address(0))
            revert TokenNotListed();

        //check if launch date of memecoin is arrived
        if (listedToken.advancedInfo.launchDate > block.timestamp)
            revert TokenNotLaunched();

        // check to ensure funding goal is not met
        if (listedToken.isFundingFinished)
            revert FundingAlreadyRaised();

        Token memeTokenCt = Token(memeTokenAddress);


        memeTokenCt.transferFrom(from, address(this), tokenQty);

        // ethAmount to send = P0 * (e^(k*c)- e^(k*(c-x))) / k
        uint tokenTotalSupply = (10000 - listedToken.advancedInfo.reservedRatio) * listedToken.advancedInfo.maxSupply / 10000;
        uint currentSupply = tokenTotalSupply - memeTokenCt.balanceOf(address(this));
        uint ethAmount = calculateCost(currentSupply - tokenQty, tokenQty, listedToken.advancedInfo.k, listedToken.advancedInfo.initialPrice);

        // decrease funding raised amount
        listedToken.fundingRaised -= ethAmount;

        (bool success, ) = payable(from).call{value: ethAmount}("");
        if (!success)
            revert FailedToSendETH();
        userBoughtAmount[memeTokenAddress][from] -= ethAmount;
        return ethAmount;
    }

    /// @notice Private function to Add liquidity on Uniswap V2 and send fees.
    /// @param memeTokenAddress The address of the meme token contract.
    function _onBondingCurveFinish(address memeTokenAddress) private {
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];

        // create liquidity pool
        address pool = _createLiquidityPool(memeTokenAddress);
    
        // provide liquidity
        uint tokenAmount = listedToken.advancedInfo.liquidityPoolRatio * listedToken.advancedInfo.maxSupply / 10000;
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

    /// @notice Private function to create a liquidity pool on Uniswap V2 for the meme token.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @return The address of the created Uniswap V2 pair.
    function _createLiquidityPool(address memeTokenAddress) private returns(address) {
        IUniswapV2Factory factory = IUniswapV2Factory(UNISWAP_V2_FACTORY_ADDRESS);
        IUniswapV2Router02 router = IUniswapV2Router02(UNISWAP_V2_ROUTER_ADDRESS);
        address pair = factory.createPair(memeTokenAddress, router.WETH());
        return pair;
    }

    /// @notice Private function to provide liquidity to the Uniswap V2 pool.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenAmount The amount of meme tokens.
    /// @param ethAmount The amount of ETH.
    /// @return The amount of liquidity provided.
    function _provideLiquidity(address memeTokenAddress, uint tokenAmount, uint ethAmount) private returns(uint){
        IERC20(memeTokenAddress).approve(UNISWAP_V2_ROUTER_ADDRESS, tokenAmount);
        IUniswapV2Router02 router = IUniswapV2Router02(UNISWAP_V2_ROUTER_ADDRESS);
        (, , uint liquidity) = router.addLiquidityETH{
            value: ethAmount
        }(memeTokenAddress, tokenAmount, tokenAmount, ethAmount, address(this), block.timestamp);
        return liquidity;
    }

    /// @notice Private function to burn the liquidity pool tokens.
    /// @param pool The address of the liquidity pool.
    /// @param liquidity The amount of liquidity to burn.
    function _burnLpTokens(address pool, uint liquidity) private {
        IUniswapV2Pair uniswapv2pairct = IUniswapV2Pair(pool);
        uniswapv2pairct.transfer(address(0), liquidity);
    }
}
