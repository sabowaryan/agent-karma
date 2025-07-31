import { Agent, Rating, Proposal, Vote, LeaderboardEntry, ApiResponse, PaginatedResponse, DashboardStats } from '../types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api';

class ApiService {
  private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`;
    
    try {
      const response = await fetch(url, {
        headers: {
          'Content-Type': 'application/json',
          ...options?.headers,
        },
        ...options,
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error(`API request failed: ${url}`, error);
      throw error;
    }
  }

  // Agent endpoints
  async getAgent(address: string): Promise<ApiResponse<Agent>> {
    return this.request<ApiResponse<Agent>>(`/agents/${address}`);
  }

  async getAgentKarma(address: string): Promise<ApiResponse<{ address: string; karma: number }>> {
    return this.request<ApiResponse<{ address: string; karma: number }>>(`/agents/${address}/karma`);
  }

  async getAgentHistory(address: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<any>> {
    return this.request<PaginatedResponse<any>>(`/agents/${address}/karma/history?page=${page}&limit=${limit}`);
  }

  async getAgentInteractions(address: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<any>> {
    return this.request<PaginatedResponse<any>>(`/agents/${address}/interactions?page=${page}&limit=${limit}`);
  }

  async getLeaderboard(page: number = 1, limit: number = 20): Promise<PaginatedResponse<LeaderboardEntry>> {
    return this.request<PaginatedResponse<LeaderboardEntry>>(`/agents?page=${page}&limit=${limit}`);
  }

  async registerAgent(metadata: Agent['metadata'], token: string): Promise<ApiResponse<{ agent: Agent; token: string }>> {
    return this.request<ApiResponse<{ agent: Agent; token: string }>>('/agents/register', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({ metadata }),
    });
  }

  // Rating endpoints
  async submitRating(ratingData: {
    ratedAddress: string;
    score: number;
    interactionHash: string;
    context?: string;
  }, token: string): Promise<ApiResponse<Rating>> {
    return this.request<ApiResponse<Rating>>('/ratings', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify(ratingData),
    });
  }

  async getRatingsForAgent(address: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<Rating>> {
    return this.request<PaginatedResponse<Rating>>(`/ratings/agent/${address}?page=${page}&limit=${limit}`);
  }

  async getRatingsByAgent(address: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<Rating>> {
    return this.request<PaginatedResponse<Rating>>(`/ratings/by/${address}?page=${page}&limit=${limit}`);
  }

  async getRecentRatings(page: number = 1, limit: number = 20): Promise<PaginatedResponse<Rating>> {
    return this.request<PaginatedResponse<Rating>>(`/ratings?page=${page}&limit=${limit}`);
  }

  async getRatingStats(address: string): Promise<ApiResponse<any>> {
    return this.request<ApiResponse<any>>(`/ratings/stats/${address}`);
  }

  // Governance endpoints
  async createProposal(proposalData: {
    title: string;
    description: string;
  }, token: string): Promise<ApiResponse<Proposal>> {
    return this.request<ApiResponse<Proposal>>('/governance/proposals', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify(proposalData),
    });
  }

  async getProposals(page: number = 1, limit: number = 20, status?: string): Promise<PaginatedResponse<Proposal>> {
    const statusParam = status ? `&status=${status}` : '';
    return this.request<PaginatedResponse<Proposal>>(`/governance/proposals?page=${page}&limit=${limit}${statusParam}`);
  }

  async getProposal(proposalId: string): Promise<ApiResponse<Proposal>> {
    return this.request<ApiResponse<Proposal>>(`/governance/proposals/${proposalId}`);
  }

  async voteOnProposal(proposalId: string, support: boolean, token: string): Promise<ApiResponse<Vote>> {
    return this.request<ApiResponse<Vote>>(`/governance/proposals/${proposalId}/vote`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
      },
      body: JSON.stringify({ support }),
    });
  }

  async getProposalVotes(proposalId: string, page: number = 1, limit: number = 20): Promise<PaginatedResponse<Vote>> {
    return this.request<PaginatedResponse<Vote>>(`/governance/proposals/${proposalId}/votes?page=${page}&limit=${limit}`);
  }

  async getGovernanceStats(): Promise<ApiResponse<any>> {
    return this.request<ApiResponse<any>>('/governance/stats');
  }

  // Dashboard-specific endpoints
  async getDashboardStats(): Promise<DashboardStats> {
    try {
      const [leaderboard, recentRatings, proposals] = await Promise.all([
        this.getLeaderboard(1, 1),
        this.getRecentRatings(1, 1),
        this.getProposals(1, 1),
        this.getGovernanceStats()
      ]);

      // Calculate stats from available data
      const stats: DashboardStats = {
        totalAgents: leaderboard.pagination?.total || 0,
        totalRatings: recentRatings.pagination?.total || 0,
        totalProposals: proposals.pagination?.total || 0,
        averageKarma: 0,
        activeAgents: 0,
        recentActivity: 0
      };

      // Get more detailed stats if available
      if (leaderboard.data && leaderboard.data.length > 0) {
        const karmaSum = leaderboard.data.reduce((sum, agent) => sum + agent.karma, 0);
        stats.averageKarma = Math.round(karmaSum / leaderboard.data.length);
      }

      return stats;
    } catch (error) {
      console.error('Failed to fetch dashboard stats', error);
      return {
        totalAgents: 0,
        totalRatings: 0,
        totalProposals: 0,
        averageKarma: 0,
        activeAgents: 0,
        recentActivity: 0
      };
    }
  }

  // Health check
  async healthCheck(): Promise<any> {
    return this.request<any>('/health', { method: 'GET' });
  }
}

export const apiService = new ApiService();
export default apiService;

