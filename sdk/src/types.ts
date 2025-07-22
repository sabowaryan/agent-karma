// Agent-Karma SDK Types
// Type definitions for the Agent-Karma system

import type { OfflineSigner } from '@cosmjs/proto-signing';
import type { ChainWalletBase } from '@cosmos-kit/core';

// Core SDK Interface
export interface IAgentKarma {
  // Connection Management
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  isConnected(): boolean;
  canSign(): boolean;

  // Agent Management
  registerAgent(metadata: AgentMetadata): Promise<string>;
  getAgent(address: string): Promise<Agent | null>;
  isAgentRegistered(address: string): Promise<boolean>;
  updateAgentMetadata(metadata: Partial<AgentMetadata>): Promise<string>;

  // Karma Operations
  getKarmaScore(address: string): Promise<KarmaScore>;
  getLeaderboard(limit?: number): Promise<LeaderboardEntry[]>;
  getKarmaHistory(address: string, limit?: number): Promise<KarmaHistoryEntry[]>;

  // Interaction Management
  logInteraction(interactionData: InteractionData): Promise<string>;
  getInteractions(agentAddress: string, limit?: number): Promise<Interaction[]>;
  getInteractionById(interactionId: string): Promise<Interaction | null>;

  // Rating System
  submitRating(ratedAgent: string, score: number, interactionHash: string, feedback?: string): Promise<string>;
  getRatings(agentAddress: string, limit?: number): Promise<Rating[]>;
  getRatingStats(agentAddress: string): Promise<RatingStats>;

  // Governance
  createProposal(proposal: ProposalData): Promise<string>;
  voteOnProposal(proposalId: string, vote: VoteOption): Promise<string>;
  getProposals(status?: ProposalStatus): Promise<Proposal[]>;
  getProposal(proposalId: string): Promise<Proposal | null>;

  // Utility Methods
  getSignerAddress(): string | undefined;
  getConfig(): AgentKarmaConfig;
  measurePerformance(): Promise<PerformanceMetric[]>;
}

// Wallet Integration Types
export interface WalletConfig {
  walletType: 'mnemonic' | 'cosmos-kit';
  mnemonic?: string;
  walletName?: string;
  signer?: OfflineSigner;
  chainWallet?: ChainWalletBase;
}

// Error Handling Types
export interface RetryConfig {
  maxRetries: number;
  baseDelay: number;
  maxDelay: number;
  backoffMultiplier: number;
}

export interface SDKError extends Error {
  code: string;
  details?: any;
  retryable: boolean;
}

export interface Agent {
  address: string;
  metadata: AgentMetadata;
  karmaScore: number;
  registrationTimestamp: number;
}

export interface AgentMetadata {
  name: string;
  description: string;
  framework: string;
  version?: string;
  capabilities?: string[];
  ipfsHash?: string;
}

export interface Rating {
  raterAddress: string;
  ratedAddress: string;
  score: number; // 1-10 scale
  interactionHash: string;
  timestamp: number;
  context?: string;
}

export interface Interaction {
  id: string;
  participants: string[];
  timestamp: number;
  hash: string;
  metadata?: Record<string, any>;
}

export interface KarmaCalculationParams {
  baseScore: number;
  timeDecayFactor: number;
  interactionBonus: number;
  contextualModifiers: Record<string, number>;
}

export interface AgentKarmaConfig {
  rpcEndpoint: string;
  chainId: string;
  contractAddresses: {
    agentRegistry: string;
    karmaCore: string;
    interactionLogger: string;
    governanceDao: string;
  };
}

export interface PerformanceMetric {
  operation: string;
  responseTime: number;
  success: boolean;
  error?: string;
}

export interface DeploymentInfo {
  network: string;
  chainId: string;
  rpcEndpoint: string;
  deployerAddress: string;
  deploymentTime: string;
  contracts: Record<string, ContractInfo>;
}

export interface ContractInfo {
  codeId: number;
  contractAddress: string;
  uploadTx: string;
  instantiateTx: string;
}

// Additional types for IAgentKarma interface
export interface KarmaScore {
  score: number;
  lastUpdated: number;
}

export interface LeaderboardEntry {
  address: string;
  karma: number;
  name: string;
  rank: number;
}

export interface KarmaHistoryEntry {
  timestamp: number;
  score: number;
  change: number;
  reason: string;
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

export interface RatingStats {
  averageScore: number;
  totalRatings: number;
  distribution: Record<number, number>;
  recentTrend: number;
}

export interface ProposalData {
  title: string;
  description: string;
  proposalType: ProposalType;
  executionData?: any;
}

export enum ProposalType {
  PARAMETER_CHANGE = 'parameter_change',
  GOVERNANCE_UPDATE = 'governance_update',
  AGENT_SUSPENSION = 'agent_suspension',
  SYSTEM_UPGRADE = 'system_upgrade'
}

export enum VoteOption {
  YES = 'yes',
  NO = 'no',
  ABSTAIN = 'abstain',
  NO_WITH_VETO = 'no_with_veto'
}

export enum ProposalStatus {
  PENDING = 'pending',
  ACTIVE = 'active',
  PASSED = 'passed',
  REJECTED = 'rejected',
  EXECUTED = 'executed'
}

export interface Proposal {
  id: string;
  title: string;
  description: string;
  proposer: string;
  status: ProposalStatus;
  proposalType: ProposalType;
  votingStartTime: number;
  votingEndTime: number;
  votes: {
    yes: number;
    no: number;
    abstain: number;
    noWithVeto: number;
  };
  executionData?: any;
}