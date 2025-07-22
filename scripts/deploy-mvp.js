#!/usr/bin/env node

/**
 * MVP Deployment Script for Agent-Karma
 * Deploys AgentRegistry, KarmaCore, and InteractionLogger contracts to Sei testnet
 */

const { CosmWasmClient, SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
const { GasPrice } = require('@cosmjs/stargate');
const fs = require('fs');
const path = require('path');

// Sei testnet configuration
const TESTNET_CONFIG = {
  rpcEndpoint: 'https://rpc.atlantic-2.seinetwork.io',
  chainId: 'atlantic-2',
  gasPrice: GasPrice.fromString('0.1usei'),
  prefix: 'sei',
};

// Contract paths (assuming compiled WASM files)
const CONTRACT_PATHS = {
  agentRegistry: path.join(__dirname, '../contracts/target/wasm32-unknown-unknown/release/agent_registry.wasm'),
  karmaCore: path.join(__dirname, '../contracts/target/wasm32-unknown-unknown/release/karma_core.wasm'),
  interactionLogger: path.join(__dirname, '../contracts/target/wasm32-unknown-unknown/release/interaction_logger.wasm'),
};

class MVPDeployer {
  constructor(mnemonic) {
    this.mnemonic = mnemonic;
    this.wallet = null;
    this.client = null;
    this.deployerAddress = null;
    this.deployedContracts = {};
  }

  async initialize() {
    console.log('🚀 Initializing MVP Deployer...');
    
    // Create wallet from mnemonic
    this.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      this.mnemonic,
      { prefix: TESTNET_CONFIG.prefix }
    );

    // Get deployer address
    const accounts = await this.wallet.getAccounts();
    this.deployerAddress = accounts[0].address;
    console.log(`📍 Deployer address: ${this.deployerAddress}`);

    // Create signing client
    this.client = await SigningCosmWasmClient.connectWithSigner(
      TESTNET_CONFIG.rpcEndpoint,
      this.wallet,
      {
        gasPrice: TESTNET_CONFIG.gasPrice,
      }
    );

    // Check balance
    const balance = await this.client.getBalance(this.deployerAddress, 'usei');
    console.log(`💰 Balance: ${balance.amount} ${balance.denom}`);

    if (parseInt(balance.amount) < 1000000) { // Less than 1 SEI
      throw new Error('Insufficient balance for deployment. Need at least 1 SEI for gas fees.');
    }
  }

  async deployContract(contractName, wasmPath, instantiateMsg) {
    console.log(`\n📦 Deploying ${contractName}...`);
    
    // Check if WASM file exists
    if (!fs.existsSync(wasmPath)) {
      throw new Error(`WASM file not found: ${wasmPath}. Please compile contracts first.`);
    }

    const wasmCode = fs.readFileSync(wasmPath);
    console.log(`📄 WASM file size: ${wasmCode.length} bytes`);

    try {
      // Upload code
      console.log(`⬆️  Uploading ${contractName} code...`);
      const uploadResult = await this.client.upload(
        this.deployerAddress,
        wasmCode,
        'auto',
        `Agent-Karma ${contractName} MVP`
      );

      console.log(`✅ Code uploaded. Code ID: ${uploadResult.codeId}`);
      console.log(`🧾 Upload transaction: ${uploadResult.transactionHash}`);

      // Instantiate contract
      console.log(`🏗️  Instantiating ${contractName}...`);
      const instantiateResult = await this.client.instantiate(
        this.deployerAddress,
        uploadResult.codeId,
        instantiateMsg,
        `agent-karma-${contractName.toLowerCase()}-mvp`,
        'auto',
        {
          admin: this.deployerAddress,
        }
      );

      console.log(`✅ ${contractName} instantiated at: ${instantiateResult.contractAddress}`);
      console.log(`🧾 Instantiate transaction: ${instantiateResult.transactionHash}`);

      this.deployedContracts[contractName] = {
        codeId: uploadResult.codeId,
        contractAddress: instantiateResult.contractAddress,
        uploadTx: uploadResult.transactionHash,
        instantiateTx: instantiateResult.transactionHash,
      };

      return instantiateResult.contractAddress;

    } catch (error) {
      console.error(`❌ Failed to deploy ${contractName}:`, error.message);
      throw error;
    }
  }

  async deployAllContracts() {
    console.log('\n🏭 Starting MVP contract deployment...\n');

    try {
      // Deploy AgentRegistry first
      const agentRegistryAddress = await this.deployContract(
        'AgentRegistry',
        CONTRACT_PATHS.agentRegistry,
        {
          admin: this.deployerAddress,
        }
      );

      // Deploy InteractionLogger
      const interactionLoggerAddress = await this.deployContract(
        'InteractionLogger',
        CONTRACT_PATHS.interactionLogger,
        {
          admin: this.deployerAddress,
        }
      );

      // Deploy KarmaCore with references to other contracts
      const karmaCoreAddress = await this.deployContract(
        'KarmaCore',
        CONTRACT_PATHS.karmaCore,
        {
          admin: this.deployerAddress,
          config: {
            min_karma_for_rating: "10",
            min_karma_for_voting: "50", 
            min_karma_for_proposal: "100",
            rating_window: 86400, // 24 hours
            max_ratings_per_interaction: 1,
            rating_fee: "2",
          },
        }
      );

      console.log('\n🎉 All contracts deployed successfully!');
      
      return {
        agentRegistry: agentRegistryAddress,
        karmaCore: karmaCoreAddress,
        interactionLogger: interactionLoggerAddress,
      };

    } catch (error) {
      console.error('\n💥 Deployment failed:', error.message);
      throw error;
    }
  }

  async saveDeploymentInfo() {
    const deploymentInfo = {
      network: 'sei-testnet',
      chainId: TESTNET_CONFIG.chainId,
      rpcEndpoint: TESTNET_CONFIG.rpcEndpoint,
      deployerAddress: this.deployerAddress,
      deploymentTime: new Date().toISOString(),
      contracts: this.deployedContracts,
    };

    const outputPath = path.join(__dirname, '../deployment-info.json');
    fs.writeFileSync(outputPath, JSON.stringify(deploymentInfo, null, 2));
    console.log(`\n📋 Deployment info saved to: ${outputPath}`);

    return deploymentInfo;
  }

  async measureTransactionTimes() {
    console.log('\n⏱️  Measuring transaction performance...');
    
    const measurements = [];
    
    // Test query performance
    for (const [contractName, info] of Object.entries(this.deployedContracts)) {
      const startTime = Date.now();
      
      try {
        // Simple query to test response time
        await this.client.queryContractSmart(info.contractAddress, { get_config: {} });
        const endTime = Date.now();
        const responseTime = endTime - startTime;
        
        measurements.push({
          contract: contractName,
          operation: 'query',
          responseTime: `${responseTime}ms`,
          success: true,
        });
        
        console.log(`📊 ${contractName} query: ${responseTime}ms`);
        
      } catch (error) {
        measurements.push({
          contract: contractName,
          operation: 'query',
          error: error.message,
          success: false,
        });
      }
    }

    return measurements;
  }
}

async function main() {
  // Get mnemonic from environment or use default test mnemonic
  const mnemonic = process.env.MNEMONIC || 
    'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

  if (mnemonic === 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about') {
    console.log('⚠️  Using default test mnemonic. Set MNEMONIC environment variable for production.');
  }

  const deployer = new MVPDeployer(mnemonic);

  try {
    await deployer.initialize();
    const contractAddresses = await deployer.deployAllContracts();
    const deploymentInfo = await deployer.saveDeploymentInfo();
    const performanceMetrics = await deployer.measureTransactionTimes();

    console.log('\n📈 Performance Summary:');
    performanceMetrics.forEach(metric => {
      if (metric.success) {
        console.log(`  ${metric.contract}: ${metric.responseTime}`);
      } else {
        console.log(`  ${metric.contract}: ERROR - ${metric.error}`);
      }
    });

    console.log('\n🎯 MVP Deployment Complete!');
    console.log('\n📋 Contract Addresses:');
    Object.entries(contractAddresses).forEach(([name, address]) => {
      console.log(`  ${name}: ${address}`);
    });

    console.log('\n🔗 Next Steps:');
    console.log('1. Update SDK configuration with deployed contract addresses');
    console.log('2. Run integration tests with: npm run test:integration');
    console.log('3. Test end-to-end flow with: npm run test:e2e');

  } catch (error) {
    console.error('\n💥 Deployment failed:', error);
    process.exit(1);
  }
}

// Run deployment if called directly
if (require.main === module) {
  main().catch(console.error);
}

module.exports = { MVPDeployer };