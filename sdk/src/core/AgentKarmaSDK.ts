// Agent-Karma SDK Core Implementation
import { StargateClient } from '@cosmjs/stargate';
import { CosmWasmClient, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import type { Agent, Rating, Interaction, AgentKarmaConfig, AgentMetadata, RetryConfig, PerformanceMetric } from '../types';
import { AgentKarmaError, withRetry } from './errors';

export interface SDKOptions {
  mnemonic?: string;
  gasPrice?: string;
  timeout?: number;
  retryConfig?: RetryConfig;
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
  private defaultRetryConfig: RetryConfig = {
    maxRetries: 3,
    baseDelay: 1000,
    maxDelay: 5000,
    backoffMultiplier: 2,
  };

  constructor(config: AgentKarmaConfig, options: SDKOptions = {}) {
    this.config = config;
    this.options = {
      gasPrice: '0.1usei',
      timeout: 30000,
      retryConfig: this.defaultRetryConfig,
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

    } catch (error: any) {
      console.error('❌ Connection failed:', error);
      throw new AgentKarmaError(`Failed to connect to Sei network: ${error.message || error}`, 'CONNECTION_FAILED', true, error);
    }
  }

  async registerAgent(metadata: AgentMetadata): Promise<string> {
    if (!this.signingClient || !this.signerAddress) {
      throw new AgentKarmaError('Signing client not available. Provide mnemonic to enable transactions.', 'SIGNING_CLIENT_UNAVAILABLE');
    }

    console.log(`📝 Registering agent: ${metadata.name}`);

    return withRetry(async () => {
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

        const result = await this.signingClient!.execute(
          this.signerAddress!,
          this.config.contractAddresses.agentRegistry,
          msg,
          'auto',
          `Register agent: ${metadata.name}`
        );

        console.log(`✅ Agent registered. Tx: ${result.transactionHash}`);
        return result.transactionHash;

      } catch (error: any) {
        console.error('❌ Agent registration failed:', error);
        throw new AgentKarmaError(`Failed to register agent: ${error.message || error}`, 'AGENT_REGISTRATION_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async logInteraction(interactionData: InteractionData): Promise<string> {
    if (!this.signingClient || !this.signerAddress) {
      throw new AgentKarmaError('Signing client not available. Provide mnemonic to enable transactions.', 'SIGNING_CLIENT_UNAVAILABLE');
    }

    console.log(`📋 Logging interaction between: ${interactionData.participants.join(', ')}`);

    return withRetry(async () => {
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

        const result = await this.signingClient!.execute(
          this.signerAddress!,
          this.config.contractAddresses.interactionLogger,
          msg,
          'auto',
          `Log interaction: ${interactionData.interactionType}`
        );

        console.log(`✅ Interaction logged. Tx: ${result.transactionHash}`);
        return result.transactionHash;

      } catch (error: any) {
        console.error('❌ Interaction logging failed:', error);
        throw new AgentKarmaError(`Failed to log interaction: ${error.message || error}`, 'INTERACTION_LOGGING_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async submitRating(
    ratedAgent: string,
    score: number,
    interactionHash: string,
    feedback?: string
  ): Promise<string> {
    if (!this.signingClient || !this.signerAddress) {
      throw new AgentKarmaError('Signing client not available. Provide mnemonic to enable transactions.', 'SIGNING_CLIENT_UNAVAILABLE');
    }

    if (score < 1 || score > 10) {
      throw new AgentKarmaError('Rating score must be between 1 and 10', 'INVALID_RATING_SCORE', false);
    }

    console.log(`⭐ Submitting rating ${score}/10 for agent: ${ratedAgent}`);

    return withRetry(async () => {
      try {
        const msg = {
          submit_rating: {
            rated_agent: ratedAgent,
            score,
            feedback,
            interaction_hash: interactionHash,
          },
        };

        const result = await this.signingClient!.execute(
          this.signerAddress!,
          this.config.contractAddresses.karmaCore,
          msg,
          'auto',
          `Submit rating: ${score}/10`
        );

        console.log(`✅ Rating submitted. Tx: ${result.transactionHash}`);
        return result.transactionHash;

      } catch (error: any) {
        console.error('❌ Rating submission failed:', error);
        throw new AgentKarmaError(`Failed to submit rating: ${error.message || error}`, 'RATING_SUBMISSION_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async getAgent(address: string): Promise<Agent | null> {
    if (!this.cosmwasmClient) {
      throw new AgentKarmaError('Client not connected. Call connect() first.', 'CLIENT_NOT_CONNECTED');
    }

    return withRetry(async () => {
      try {
        const response = await this.cosmwasmClient!.queryContractSmart(
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

      } catch (error: any) {
        console.error('❌ Failed to get agent:', error);
        throw new AgentKarmaError(`Failed to get agent: ${error.message || error}`, 'GET_AGENT_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async getKarmaScore(address: string): Promise<KarmaScore> {
    if (!this.cosmwasmClient) {
      throw new AgentKarmaError('Client not connected. Call connect() first.', 'CLIENT_NOT_CONNECTED');
    }

    return withRetry(async () => {
      try {
        const response = await this.cosmwasmClient!.queryContractSmart(
          this.config.contractAddresses.karmaCore,
          {
            get_karma_score: { agent_address: address },
          }
        );

        return {
          score: parseInt(response.score),
          lastUpdated: response.last_updated,
        };

      } catch (error: any) {
        console.error('❌ Failed to get karma score:', error);
        throw new AgentKarmaError(`Failed to get karma score: ${error.message || error}`, 'GET_KARMA_SCORE_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async getInteractions(agentAddress: string, limit: number = 10): Promise<Interaction[]> {
    if (!this.cosmwasmClient) {
      throw new AgentKarmaError('Client not connected. Call connect() first.', 'CLIENT_NOT_CONNECTED');
    }

    return withRetry(async () => {
      try {
        const response = await this.cosmwasmClient!.queryContractSmart(
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

      } catch (error: any) {
        console.error('❌ Failed to get interactions:', error);
        throw new AgentKarmaError(`Failed to get interactions: ${error.message || error}`, 'GET_INTERACTIONS_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async getLeaderboard(limit: number = 20): Promise<Array<{ address: string; karma: number; name: string }>> {
    if (!this.cosmwasmClient) {
      throw new AgentKarmaError('Client not connected. Call connect() first.', 'CLIENT_NOT_CONNECTED');
    }

    return withRetry(async () => {
      try {
        const response = await this.cosmwasmClient!.queryContractSmart(
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

      } catch (error: any) {
        console.error('❌ Failed to get leaderboard:', error);
        throw new AgentKarmaError(`Failed to get leaderboard: ${error.message || error}`, 'GET_LEADERBOARD_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async isAgentRegistered(address: string): Promise<boolean> {
    if (!this.cosmwasmClient) {
      throw new AgentKarmaError('Client not connected. Call connect() first.', 'CLIENT_NOT_CONNECTED');
    }

    return withRetry(async () => {
      try {
        const response = await this.cosmwasmClient!.queryContractSmart(
          this.config.contractAddresses.agentRegistry,
          {
            is_registered: { agent_address: address },
          }
        );

        return response.registered;

      } catch (error: any) {
        console.error('❌ Failed to check registration:', error);
        throw new AgentKarmaError(`Failed to check registration: ${error.message || error}`, 'IS_REGISTERED_FAILED', true, error);
      }
    }, this.options.retryConfig!);
  }

  async measurePerformance(): Promise<PerformanceMetric[]> {
    const measurements: PerformanceMetric[] = [];

    if (!this.cosmwasmClient) {
      throw new AgentKarmaError('Client not connected. Call connect() first.', 'CLIENT_NOT_CONNECTED');
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
          success: true,
        });
      } catch (error: any) {
        console.warn(`⚠️  Performance test failed for ${operation.name}:`, error);
        measurements.push({
          operation: operation.name,
          responseTime: -1, // Indicate failure
          success: false,
          error: error.message || String(error),
        });
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

