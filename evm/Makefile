-include .env

build:; forge build

deploy-local:
	$(MAKE) deploy RPC_URL=localhost EXTRA_FLAGS=

deploy-bnbTestnet:
	$(MAKE) deploy-bnb RPC_URL=bnb_test EXTRA_FLAGS="--verify --etherscan-api-key ${BNB_TEST_ETHERSCAN_API_KEY} --verifier-url ${BNB_TEST_ETHERSCAN_API_URL} --slow"

deploy-baseTestnet:
	$(MAKE) deploy-base RPC_URL=base_sepolia EXTRA_FLAGS="--verify --etherscan-api-key ${BASE_ETHERSCAN_API_KEY} --verifier-url ${BASE_SEPOLIA_ETHERSCAN_API_URL} --slow"

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