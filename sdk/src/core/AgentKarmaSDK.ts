// Agent-Karma SDK Core Implementation
import { StargateClient } from '@cosmjs/stargate';
import { CosmWasmClient, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import type { Agent, Rating, Interaction, AgentKarmaConfig, AgentMetadata } from '../types';

export interface SDKOptions {
  mnemonic?: string;
  gasPrice?: string;
  timeout?: number;
}

export interface KarmaScore {
  score: number;
  lastUpdated: number;
}

export interface InteractionData {
  participants: string[];
  interactionType: string;
  metadata?: {
    duration?: number;
    outcome?: string;
    context?: string;
  };
}

export class AgentKarmaSDK {
  private config: AgentKarmaConfig;
  private stargateClient?: StargateClient;
  private cosmwasmClient?: CosmWasmClient;
  private signingClient?: SigningCosmWasmClient;
  private wallet?: DirectSecp256k1HdWallet;
  private signerAddress?: string;
  private options: SDKOptions;

  constructor(config: AgentKarmaConfig, options: SDKOptions = {}) {
    this.config = config;
    this.options = {
      gasPrice: '0.1usei',
      timeout: 30000,
      ...options,
    };
  }

  async connect(): Promise<void> {
    console.log('🔌 Connecting to Sei network...');
    
    try {
      // Connect read-only clients
      this.stargateClient = await StargateClient.connect(this.config.rpcEndpoint);
      this.cosmwasmClient = await CosmWasmClient.connect(this.config.rpcEndpoint);

      // Setup signing client if mnemonic provided
      if (this.options.mnemonic) {
        this.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
          this.options.mnemonic,
          { prefix: 'sei' }
        );

        const accounts = await this.wallet.getAccounts();
        this.signerAddress = accounts[0].address;

        this.signingClient = await SigningCosmWasmClient.connectWithSigner(
          this.config.rpcEndpoint,
          this.wallet,
          {
            gasPrice: GasPrice.fromString(this.options.gasPrice!),
          }
        );

        console.log(`✅ Connected as: ${this.signerAddress}`);
      } else {
        console.log('✅ Connected in read-only mode');
      }

    } catch (error) {
      console.error('❌ Connection failed:', error);
      throw new Error(`Failed to connect to Sei network: ${error}`);
    }
  }

  async registerAgent(metadata: AgentMetadata): Promise<string> {
    if (!this.signingClient || !this.signerAddress) {
      throw new Error('Signing client not available. Provide mnemonic to enable transactions.');
    }

    console.log(`📝 Registering agent: ${metadata.name}`);

    try {
      const msg = {
        register_agent: {
          metadata: {
            name: metadata.name,
            description: metadata.description,
            framework: metadata.framework || 'Custom',
            version: metadata.version || '1.0.0',
            ipfs_hash: metadata.ipfsHash,
          },
        },
      };

      const result = await this.signingClient.execute(
        this.signerAddress,
        this.config.contractAddresses.agentRegistry,
        msg,
        'auto',
        `Register agent: ${metadata.name}`
      );

      console.log(`✅ Agent registered. Tx: ${result.transactionHash}`);
      return result.transactionHash;

    } catch (error) {
      console.error('❌ Agent registration failed:', error);
      throw new Error(`Failed to register agent: ${error}`);
    }
  }

  async logInteraction(interactionData: InteractionData): Promise<string> {
    if (!this.signingClient || !this.signerAddress) {
      throw new Error('Signing client not available. Provide mnemonic to enable transactions.');
    }

    console.log(`📋 Logging interaction between: ${interactionData.participants.join(', ')}`);

    try {
      const msg = {
        log_interaction: {
          participants: interactionData.participants,
          interaction_type: interactionData.interactionType,
          metadata: {
            duration: interactionData.metadata?.duration,
            outcome: interactionData.metadata?.outcome,
            context: interactionData.metadata?.context,
          },
        },
      };

      const result = await this.signingClient.execute(
        this.signerAddress,
        this.config.contractAddresses.interactionLogger,
        msg,
        'auto',
        `Log interaction: ${interactionData.interactionType}`
      );

      console.log(`✅ Interaction logged. Tx: ${result.transactionHash}`);
      return result.transactionHash;

    } catch (error) {
      console.error('❌ Interaction logging failed:', error);
      throw new Error(`Failed to log interaction: ${error}`);
    }
  }

  async submitRating(
    ratedAgent: string,
    score: number,
    interactionHash: string,
    feedback?: string
  ): Promise<string> {
    if (!this.signingClient || !this.signerAddress) {
      throw new Error('Signing client not available. Provide mnemonic to enable transactions.');
    }

    if (score < 1 || score > 10) {
      throw new Error('Rating score must be between 1 and 10');
    }

    console.log(`⭐ Submitting rating ${score}/10 for agent: ${ratedAgent}`);

    try {
      const msg = {
        submit_rating: {
          rated_agent: ratedAgent,
          score,
          feedback,
          interaction_hash: interactionHash,
        },
      };

      const result = await this.signingClient.execute(
        this.signerAddress,
        this.config.contractAddresses.karmaCore,
        msg,
        'auto',
        `Submit rating: ${score}/10`
      );

      console.log(`✅ Rating submitted. Tx: ${result.transactionHash}`);
      return result.transactionHash;

    } catch (error) {
      console.error('❌ Rating submission failed:', error);
      throw new Error(`Failed to submit rating: ${error}`);
    }
  }

  async getAgent(address: string): Promise<Agent | null> {
    if (!this.cosmwasmClient) {
      throw new Error('Client not connected. Call connect() first.');
    }

    try {
      const response = await this.cosmwasmClient.queryContractSmart(
        this.config.contractAddresses.agentRegistry,
        {
          get_agent: { agent_address: address },
        }
      );

      if (!response.agent) {
        return null;
      }

      return {
        address: response.agent.address,
        metadata: {
          name: response.agent.metadata.name,
          description: response.agent.metadata.description,
          version: response.agent.metadata.version,
          capabilities: response.agent.metadata.capabilities,
          ipfsHash: response.agent.metadata.ipfs_hash,
          framework: response.agent.metadata.framework,
        },
        karmaScore: parseInt(response.agent.karma_score),
        registrationTimestamp: response.agent.registration_date,
      };

    } catch (error) {
      console.error('❌ Failed to get agent:', error);
      return null;
    }
  }

  async getKarmaScore(address: string): Promise<KarmaScore> {
    if (!this.cosmwasmClient) {
      throw new Error('Client not connected. Call connect() first.');
    }

    try {
      const response = await this.cosmwasmClient.queryContractSmart(
        this.config.contractAddresses.karmaCore,
        {
          get_karma_score: { agent_address: address },
        }
      );

      return {
        score: parseInt(response.score),
        lastUpdated: response.last_updated,
      };

    } catch (error) {
      console.error('❌ Failed to get karma score:', error);
      return { score: 0, lastUpdated: 0 };
    }
  }

  async getInteractions(agentAddress: string, limit: number = 10): Promise<Interaction[]> {
    if (!this.cosmwasmClient) {
      throw new Error('Client not connected. Call connect() first.');
    }

    try {
      const response = await this.cosmwasmClient.queryContractSmart(
        this.config.contractAddresses.interactionLogger,
        {
          get_interaction_history: {
            agent_address: agentAddress,
            limit,
          },
        }
      );

      return response.interactions.map((interaction: any) => ({
        id: interaction.id,
        participants: interaction.participants,
        timestamp: interaction.timestamp,
        hash: interaction.hash || '',
        metadata: interaction.metadata,
      }));

    } catch (error) {
      console.error('❌ Failed to get interactions:', error);
      return [];
    }
  }

  async getLeaderboard(limit: number = 20): Promise<Array<{ address: string; karma: number; name: string }>> {
    if (!this.cosmwasmClient) {
      throw new Error('Client not connected. Call connect() first.');
    }

    try {
      const response = await this.cosmwasmClient.queryContractSmart(
        this.config.contractAddresses.karmaCore,
        {
          get_leaderboard: { limit },
        }
      );

      return response.leaderboard.map((entry: any) => ({
        address: entry.agent_address,
        karma: parseInt(entry.karma_score),
        name: entry.agent_name || 'Unknown',
      }));

    } catch (error) {
      console.error('❌ Failed to get leaderboard:', error);
      return [];
    }
  }

  async isAgentRegistered(address: string): Promise<boolean> {
    if (!this.cosmwasmClient) {
      throw new Error('Client not connected. Call connect() first.');
    }

    try {
      const response = await this.cosmwasmClient.queryContractSmart(
        this.config.contractAddresses.agentRegistry,
        {
          is_registered: { agent_address: address },
        }
      );

      return response.registered;

    } catch (error) {
      console.error('❌ Failed to check registration:', error);
      return false;
    }
  }

  async measurePerformance(): Promise<{ operation: string; responseTime: number }[]> {
    const measurements: { operation: string; responseTime: number }[] = [];

    if (!this.cosmwasmClient) {
      throw new Error('Client not connected. Call connect() first.');
    }

    // Test various operations
    const operations = [
      {
        name: 'getKarmaScore',
        fn: () => this.getKarmaScore(this.signerAddress || 'sei1test'),
      },
      {
        name: 'isAgentRegistered',
        fn: () => this.isAgentRegistered(this.signerAddress || 'sei1test'),
      },
      {
        name: 'getLeaderboard',
        fn: () => this.getLeaderboard(5),
      },
    ];

    for (const operation of operations) {
      const startTime = Date.now();
      try {
        await operation.fn();
        const endTime = Date.now();
        measurements.push({
          operation: operation.name,
          responseTime: endTime - startTime,
        });
      } catch (error) {
        console.warn(`⚠️  Performance test failed for ${operation.name}:`, error);
      }
    }

    return measurements;
  }

  async disconnect(): Promise<void> {
    console.log('🔌 Disconnecting from Sei network...');
    
    // Cleanup connections
    this.stargateClient = undefined;
    this.cosmwasmClient = undefined;
    this.signingClient = undefined;
    this.wallet = undefined;
    this.signerAddress = undefined;
    
    console.log('✅ Disconnected');
  }

  // Utility methods
  getSignerAddress(): string | undefined {
    return this.signerAddress;
  }

  getConfig(): AgentKarmaConfig {
    return { ...this.config };
  }

  isConnected(): boolean {
    return !!this.cosmwasmClient;
  }

  canSign(): boolean {
    return !!this.signingClient && !!this.signerAddress;
  }
}