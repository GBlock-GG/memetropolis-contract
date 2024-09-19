// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.26;

import "./Token.sol";
import "@uniswap-v2-core-1.0.1/contracts/interfaces/IUniswapV2Factory.sol";
import "@uniswap-v2-core-1.0.1/contracts/interfaces/IUniswapV2Pair.sol";
import "@uniswap-v2-periphery-1.1.0-beta.0/contracts/interfaces/IUniswapV2Router02.sol";
import "@openzeppelin-contracts-5.0.2/utils/ReentrancyGuard.sol";
import "@openzeppelin-contracts-5.0.2/access/Ownable.sol";
import {console} from "forge-std/console.sol";

/// @title Meme Token Factory
/// @notice This contract allows users to create and manage Meme tokens, which can be traded and sold on Uniswap.
/// @dev It includes features such as funding, token creation, Uniswap liquidity pool creation, and trading based on an exponential bonding curve.
contract TokenFactory is ReentrancyGuard, Ownable {

    /// @notice Data structure to hold information about each meme token.
    struct memeToken {
        string name;
        string symbol;
        string description;
        string tokenImageUrl;
        uint fundingRaised;
        address tokenAddress;
        address creatorAddress;
    }

    /// @notice Array to store the addresses of all created meme tokens.
    address[] public memeTokenAddresses;

    /// @notice Maps meme token address to its associated details in `memeToken` struct.
    mapping(address => memeToken) public addressToMemeTokenMapping;

    // uint constant MEMETOKEN_CREATION_PLATFORM_FEE = 0.0001 ether;
    uint constant MEMECOIN_FUNDING_GOAL = 24 ether;
    uint constant MEMECOIN_ETH_TO_DEPOSIT = 5 ether;

    address constant UNISWAP_V2_FACTORY_ADDRESS = 0x8909Dc15e40173Ff4699343b6eB8132c65e18eC6;
    address constant UNISWAP_V2_ROUTER_ADDRESS = 0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24;


    uint constant DECIMALS = 10 ** 18;
    uint public constant MAX_SUPPLY = 1000000 * DECIMALS;
    uint public constant INIT_SUPPLY = 20 * MAX_SUPPLY / 100;

    uint256 public constant INITIAL_PRICE = 30000000000000;  // Initial price in wei (P0), 3.00 * 10^13
    uint256 public constant K = 8 * 10**15;  // Growth rate (k), scaled to avoid precision loss (0.01 * 10^18)

    event CreatedMemeToken(address indexed tokenAddress, address indexed creator, string name, string symbol);
    event BoughtMemeToken(address indexed memeTokenAddress, address indexed user, uint tokenQty);
    event SoldMemeToken(address indexed memeTokenAddress, address indexed user, uint tokenQty);
    event WithdrawnETH(uint256 amount);
    event WithdrawnToken(address tokenAddress, uint256 amount);

    constructor(address initialOwner)
        Ownable(initialOwner)
    {}


    /// @notice Calculates the cost in wei for purchasing tokens using an exponential bonding curve.
    /// @param currentSupply The current token supply.
    /// @param tokensToBuy The number of tokens the user wants to buy.
    /// @return The total cost in wei required to purchase the tokens.
    function calculateCost(uint256 currentSupply, uint256 tokensToBuy) public pure returns (uint256) {
        
        // Calculate the exponent parts scaled to avoid precision loss
        uint256 exponent1 = (K * (currentSupply + tokensToBuy)) / 10**18;
        uint256 exponent2 = (K * currentSupply) / 10**18;

        // Calculate e^(kx) using the exp function
        uint256 exp1 = exp(exponent1);
        uint256 exp2 = exp(exponent2);

        // Cost formula: (P0 / k) * (e^(k * (currentSupply + tokensToBuy)) - e^(k * currentSupply))
        // We use (P0 * 10^18) / k to keep the division safe from zero
        uint256 cost = (INITIAL_PRICE * 10**18 * (exp1 - exp2)) / K;  // Adjust for k scaling without dividing by zero
        return cost;
    }

    /// @notice Helper function to approximate e^x using a Taylor series expansion.
    /// @param x The exponent value.
    /// @return The approximated result of e^x.
    function exp(uint256 x) internal pure returns (uint256) {
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

    /// @notice Creates a new Meme Token.
    /// @param name The name of the meme token.
    /// @param symbol The symbol of the meme token.
    /// @param imageUrl The image URL for the meme token.
    /// @param description The description for the meme token.
    /// @return The address of the created meme token contract.
    function createMemeToken(string memory name, string memory symbol, string memory imageUrl, string memory description) public returns(address) {

        //should deploy the meme token, mint the initial supply to the token factory contract
        // require(msg.value>= MEMETOKEN_CREATION_PLATFORM_FEE, "fee not paid for memetoken creation");
        Token ct = new Token(name, symbol, INIT_SUPPLY);
        address memeTokenAddress = address(ct);
        memeToken memory newlyCreatedToken = memeToken(name, symbol, description, imageUrl, 0, memeTokenAddress, msg.sender);
        memeTokenAddresses.push(memeTokenAddress);
        addressToMemeTokenMapping[memeTokenAddress] = newlyCreatedToken;

        emit CreatedMemeToken(memeTokenAddress, msg.sender, name, symbol);

        return memeTokenAddress;
    }

    /// @notice Retrieves all created Meme Tokens.
    /// @return An array of memeToken structs representing all created meme tokens.
    function getAllMemeTokens() public view returns(memeToken[] memory) {
        memeToken[] memory allTokens = new memeToken[](memeTokenAddresses.length);
        for (uint i = 0; i < memeTokenAddresses.length; i++) {
            allTokens[i] = addressToMemeTokenMapping[memeTokenAddresses[i]];
        }
        return allTokens;
    }

    /// @notice Calculates the required ETH for purchasing a certain quantity of meme tokens.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The number of tokens to buy.
    /// @return The required ETH in wei to purchase the given quantity of tokens.
    function getRequiredEth(address memeTokenAddress, uint tokenQty) public view returns (uint) {
        //check if memecoin is listed
        require(addressToMemeTokenMapping[memeTokenAddress].tokenAddress!=address(0), "Token is not listed");

        Token memeTokenCt = Token(memeTokenAddress);
        uint currentSupply = memeTokenCt.totalSupply();
        uint currentSupplyScaled = (currentSupply - INIT_SUPPLY) / DECIMALS;
        uint requiredEth = calculateCost(currentSupplyScaled, tokenQty);

        return requiredEth;
    }

    /// @notice Allows users to buy meme tokens using ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The number of tokens to buy.
    /// @return The result of the purchase.
    function buyMemeToken(address memeTokenAddress, uint tokenQty) public payable returns(uint) {

        //check if memecoin is listed
        require(addressToMemeTokenMapping[memeTokenAddress].tokenAddress!=address(0), "Token is not listed");
        
        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];


        Token memeTokenCt = Token(memeTokenAddress);

        // check to ensure funding goal is not met
        require(listedToken.fundingRaised <= MEMECOIN_FUNDING_GOAL, "Funding has already been raised");


        // check to ensure there is enough supply to facilitate the purchase
        uint currentSupply = memeTokenCt.totalSupply();
        console.log("Current supply of token is ", currentSupply);
        console.log("Max supply of token is ", MAX_SUPPLY);
        uint available_qty = MAX_SUPPLY - currentSupply;
        console.log("Qty available for purchase ",available_qty);


        uint scaled_available_qty = available_qty / DECIMALS;
        uint tokenQty_scaled = tokenQty * DECIMALS;

        require(tokenQty <= scaled_available_qty, "Not enough available supply");

        // calculate the cost for purchasing tokenQty tokens as per the exponential bonding curve formula
        uint currentSupplyScaled = (currentSupply - INIT_SUPPLY) / DECIMALS;
        uint requiredEth = calculateCost(currentSupplyScaled, tokenQty);

        console.log("ETH required for purchasing meme tokens is ",requiredEth);

        // check if user has sent correct value of eth to facilitate this purchase
        require(msg.value >= requiredEth, "Incorrect value of ETH sent");

        // Incerement the funding
        listedToken.fundingRaised+= msg.value;

        if(listedToken.fundingRaised >= MEMECOIN_FUNDING_GOAL){
            // create liquidity pool
            address pool = _createLiquidityPool(memeTokenAddress);
            console.log("Pool address ", pool);

            // provide liquidity
            uint tokenAmount = INIT_SUPPLY;
            uint ethAmount = MEMECOIN_ETH_TO_DEPOSIT;
            uint liquidity = _provideLiquidity(memeTokenAddress, tokenAmount, ethAmount);
            console.log("Uniswap provided liquidty ", liquidity);

            // burn lp token
            _burnLpTokens(pool, liquidity);

        }

        // mint the tokens
        memeTokenCt.mint(tokenQty_scaled, msg.sender);

        console.log("User balance of the tokens is ", memeTokenCt.balanceOf(msg.sender));

        console.log("New available qty ", MAX_SUPPLY - memeTokenCt.totalSupply());

        emit BoughtMemeToken(memeTokenAddress, msg.sender, tokenQty);

        return 1;
    }

    /// @notice Allows users to sell meme tokens for ETH.
    /// @param memeTokenAddress The address of the meme token contract.
    /// @param tokenQty The number of tokens to sell.
    /// @return The amount of ETH received in return.
    function sellMemeToken(address memeTokenAddress, uint tokenQty) public nonReentrant returns(uint) {
        Token memeTokenCt = Token(memeTokenAddress);

        require(memeTokenCt.balanceOf(address(msg.sender)) >= tokenQty, "Insufficient token amount.");

        memeToken storage listedToken = addressToMemeTokenMapping[memeTokenAddress];

        // check to ensure funding goal is not met
        require(listedToken.fundingRaised <= MEMECOIN_FUNDING_GOAL, "Funding has already been raised.");

        memeTokenCt.transferFrom(msg.sender, address(this), tokenQty);

        uint ethAmount = tokenQty * INITIAL_PRICE / DECIMALS;
        (bool success, ) = payable(msg.sender).call{value: ethAmount}("");
        require(success, "Failed to send Ether");

        emit SoldMemeToken(memeTokenAddress, msg.sender, tokenQty);

        return ethAmount;
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
        Token memeTokenCt = Token(memeTokenAddress);
        memeTokenCt.approve(UNISWAP_V2_ROUTER_ADDRESS, tokenAmount);
        IUniswapV2Router02 router = IUniswapV2Router02(UNISWAP_V2_ROUTER_ADDRESS);
        (, , uint liquidity) = router.addLiquidityETH{
            value: ethAmount
        }(memeTokenAddress, tokenAmount, tokenAmount, ethAmount, address(this), block.timestamp);
        return liquidity;
    }

    /// @notice Internal function to burn the liquidity pool tokens.
    /// @param pool The address of the liquidity pool.
    /// @param liquidity The amount of liquidity to burn.
    /// @return The result of the burning operation.
    function _burnLpTokens(address pool, uint liquidity) internal returns(uint){
        IUniswapV2Pair uniswapv2pairct = IUniswapV2Pair(pool);
        uniswapv2pairct.transfer(address(0), liquidity);
        console.log("Uni v2 tokens burnt");
        return 1;
    }

    /// @notice Allows the owner to withdraw ETH from the contract
    /// @param amount The amount of ETH to withdraw
    /// @dev Emits a WithdrawnETH event on successful withdrawal
    function withdrawETH(uint256 amount) external onlyOwner {
        require(address(this).balance >= amount, "Insufficient balance");
        
        (bool success, ) = payable(owner()).call{value: amount}("");
        require(success, "Failed to send Ether");
        emit WithdrawnETH(amount);
    }

    /// @notice Allows the owner to withdraw ERC20 token from the contract
    /// @param amount The amount of token to withdraw
    /// @dev Emits a WithdrawnToken event on successful withdrawal
    function withdrawTokens(address tokenAddress, uint256 amount) external onlyOwner {
        require(IERC20(tokenAddress).balanceOf(address(this)) >= amount, "Insufficient token balance");
        IERC20(tokenAddress).transfer(owner(), amount);

        emit WithdrawnToken(tokenAddress, amount);
    }

    /// @notice Fallback function to allow the contract to receive ETH.
    receive() external payable {}
}
