-include .env

build:; forge build

deploy-local:
	$(MAKE) deploy RPC_URL=localhost EXTRA_FLAGS=

deploy-base:
	$(MAKE) deploy RPC_URL=base EXTRA_FLAGS="--verify --etherscan-api-key ${BASE_ETHERSCAN_API_KEY} --verifier-url ${BASE_ETHERSCAN_API_URL} --slow"

deploy:
	@echo "Running deploy '$(RPC_URL)'"
	@rm -rf out && \
	forge script script/DeployTokenFactory.s.sol --rpc-url $(RPC_URL) \
		--private-key $(DEPLOYER_PRIVATE_KEY) --broadcast $(EXTRA_FLAGS) -vvvv
