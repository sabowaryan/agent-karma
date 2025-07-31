// Types for Agent-Karma Dashboard

export interface Agent {
  address: string;
  metadata: {
    name: string;
    description?: string;
    capabilities?: string[];
    ipfsHash?: string;
  };
  karma: number;
  registrationDate: string;
  lastUpdate: string;
  interactionCount: number;
}

export interface Rating {
  id: string;
  raterAddress: string;
  ratedAddress: string;
  score: number;
  interactionHash: string;
  context?: string;
  timestamp: string;
  blockHeight: number;
}

export interface Interaction {
  id: string;
  hash: string;
  participants: string[];
  type: string;
  timestamp: string;
  blockHeight: number;
  metadata?: Record<string, any>;
}

export interface Proposal {
  id: string;
  title: string;
  description: string;
  proposer: string;
  status: 'active' | 'passed' | 'rejected' | 'executed';
  votesFor: number;
  votesAgainst: number;
  quorum: number;
  deadline: string;
  createdAt: string;
  executedAt?: string;
}

export interface Vote {
  proposalId: string;
  voter: string;
  support: boolean;
  votingPower: number;
  timestamp: string;
}

export interface KarmaHistory {
  address: string;
  score: number;
  timestamp: string;
  blockHeight: number;
  reason: string;
}

export interface LeaderboardEntry {
  address: string;
  karma: number;
  rank: number;
  metadata: {
    name: string;
    description?: string;
  };
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

export interface PaginatedResponse<T> {
  success: boolean;
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
  timestamp: string;
}

export interface WebSocketEvent {
  type: 'karma_updated' | 'rating_submitted' | 'agent_registered' | 'proposal_created' | 'vote_cast';
  data: any;
  timestamp: string;
}

export interface DashboardStats {
  totalAgents: number;
  totalRatings: number;
  totalProposals: number;
  averageKarma: number;
  activeAgents: number;
  recentActivity: number;
}

export interface ChartData {
  labels: string[];
  datasets: {
    label: string;
    data: number[];
    backgroundColor?: string | string[];
    borderColor?: string | string[];
    borderWidth?: number;
  }[];
}

export interface FilterOptions {
  sortBy: 'karma' | 'interactions' | 'registrationDate';
  sortOrder: 'asc' | 'desc';
  minKarma?: number;
  maxKarma?: number;
  capabilities?: string[];
  timeRange?: 'day' | 'week' | 'month' | 'year' | 'all';
}

