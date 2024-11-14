-include .env

build:; forge build

deploy-local:
	$(MAKE) deploy RPC_URL=localhost EXTRA_FLAGS=

deploy-bnbTestnet:
	$(MAKE) deploy-bnb RPC_URL=bnb_test EXTRA_FLAGS="--verify --etherscan-api-key ${BNB_TEST_ETHERSCAN_API_KEY} --verifier-url ${BNB_TEST_ETHERSCAN_API_URL} --slow"

deploy-baseTestnet:
	$(MAKE) deploy-base RPC_URL=base_sepolia EXTRA_FLAGS="--verify --etherscan-api-key ${BASE_ETHERSCAN_API_KEY} --verifier-url ${BASE_SEPOLIA_ETHERSCAN_API_URL} --slow"

deploy-arbitrumTestnet:
	$(MAKE) deploy-arbitrum RPC_URL=arbitrum_sepolia EXTRA_FLAGS="--verify --etherscan-api-key ${ARBITRUM_ETHERSCAN_API_KEY} --verifier-url ${ARBITRUM_SEPOLIA_ETHERSCAN_API_URL} --slow"

deploy-ethTestnet:
	$(MAKE) deploy-eth RPC_URL=eth_sepolia EXTRA_FLAGS="--verify --etherscan-api-key ${ETH_ETHERSCAN_API_KEY} --verifier-url ${ETH_SEPOLIA_ETHERSCAN_API_URL} --slow"

deploy-base:
	@echo "Running deploy '$(RPC_URL)'"
	@rm -rf out && \
	forge script script/base/DeployTokenFactory.s.sol --rpc-url $(RPC_URL) \
		--private-key $(DEPLOYER_PRIVATE_KEY) --broadcast $(EXTRA_FLAGS) -vvvv

deploy-bnb:
	@echo "Running deploy '$(RPC_URL)'"
	@rm -rf out && \
	forge script script/bnb/DeployTokenFactory.s.sol --rpc-url $(RPC_URL) \
		--private-key $(DEPLOYER_PRIVATE_KEY) --broadcast $(EXTRA_FLAGS) -vvvv

deploy-arbitrum:
	@echo "Running deploy '$(RPC_URL)'"
	@rm -rf out && \
	forge script script/arbitrum/DeployTokenFactory.s.sol --rpc-url $(RPC_URL) \
		--private-key $(DEPLOYER_PRIVATE_KEY) --broadcast $(EXTRA_FLAGS) -vvvv

deploy-eth:
	@echo "Running deploy '$(RPC_URL)'"
	@rm -rf out && \
	forge script script/eth/DeployTokenFactory.s.sol --rpc-url $(RPC_URL) \
		--private-key $(DEPLOYER_PRIVATE_KEY) --broadcast $(EXTRA_FLAGS) -vvvv

verify-bnbToken:
	@echo "Running verify "
	@rm -rf out && \
	forge verify-contract  \
		--etherscan-api-key ${BNB_TEST_ETHERSCAN_API_KEY} \
		--verifier-url ${BNB_TEST_ETHERSCAN_API_URL} \
		--constructor-args-path constructor-args/Token.txt \
		0xF37Ca447b60125074e03Ed26772cAEDDE6FEEA3e \
		src/Token.sol:Token
