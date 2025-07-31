import React, { useState, useEffect } from 'react';
import { DashboardStats, LeaderboardEntry, Rating, Proposal } from '../../types';
import { apiService } from '../../services/api';
import { webSocketService } from '../../services/websocket';
import StatsCards from './StatsCards';
import Leaderboard from './Leaderboard';
import RecentActivity from './RecentActivity';
import ProposalsList from './ProposalsList';
import KarmaChart from './KarmaChart';
import './Dashboard.css';

const Dashboard: React.FC = () => {
  const [stats, setStats] = useState<DashboardStats>({
    totalAgents: 0,
    totalRatings: 0,
    totalProposals: 0,
    averageKarma: 0,
    activeAgents: 0,
    recentActivity: 0
  });
  
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [recentRatings, setRecentRatings] = useState<Rating[]>([]);
  const [activeProposals, setActiveProposals] = useState<Proposal[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [wsConnected, setWsConnected] = useState(false);

  useEffect(() => {
    loadDashboardData();
    setupWebSocket();

    return () => {
      webSocketService.disconnect();
    };
  }, []);

  const loadDashboardData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Load all dashboard data in parallel
      const [statsData, leaderboardData, ratingsData, proposalsData] = await Promise.all([
        apiService.getDashboardStats(),
        apiService.getLeaderboard(1, 10),
        apiService.getRecentRatings(1, 10),
        apiService.getProposals(1, 5, 'active')
      ]);

      setStats(statsData);
      
      if (leaderboardData.success && leaderboardData.data) {
        setLeaderboard(leaderboardData.data);
      }
      
      if (ratingsData.success && ratingsData.data) {
        setRecentRatings(ratingsData.data);
      }
      
      if (proposalsData.success && proposalsData.data) {
        setActiveProposals(proposalsData.data);
      }

    } catch (err) {
      console.error('Failed to load dashboard data:', err);
      setError('Failed to load dashboard data. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const setupWebSocket = () => {
    webSocketService.connect();
    
    // Subscribe to all event types for dashboard updates
    webSocketService.subscribe([
      'karma_updated',
      'rating_submitted',
      'agent_registered',
      'proposal_created',
      'vote_cast'
    ]);

    // Add event listeners for real-time updates
    webSocketService.addEventListener('karma_updated', handleKarmaUpdate);
    webSocketService.addEventListener('rating_submitted', handleRatingSubmitted);
    webSocketService.addEventListener('agent_registered', handleAgentRegistered);
    webSocketService.addEventListener('proposal_created', handleProposalCreated);
    webSocketService.addEventListener('vote_cast', handleVoteCast);

    // Monitor connection status
    const checkConnection = () => {
      setWsConnected(webSocketService.getConnectionStatus());
    };

    const connectionInterval = setInterval(checkConnection, 5000);
    checkConnection();

    return () => {
      clearInterval(connectionInterval);
      webSocketService.removeEventListener('karma_updated', handleKarmaUpdate);
      webSocketService.removeEventListener('rating_submitted', handleRatingSubmitted);
      webSocketService.removeEventListener('agent_registered', handleAgentRegistered);
      webSocketService.removeEventListener('proposal_created', handleProposalCreated);
      webSocketService.removeEventListener('vote_cast', handleVoteCast);
    };
  };

  const handleKarmaUpdate = (data: any) => {
    console.log('Karma updated:', data);
    // Update leaderboard if the agent is in current view
    setLeaderboard(prev => 
      prev.map(agent => 
        agent.address === data.agentAddress 
          ? { ...agent, karma: data.newKarma }
          : agent
      )
    );
    
    // Update stats
    setStats(prev => ({
      ...prev,
      recentActivity: prev.recentActivity + 1
    }));
  };

  const handleRatingSubmitted = (data: any) => {
    console.log('Rating submitted:', data);
    // Add to recent ratings
    setRecentRatings(prev => [data, ...prev.slice(0, 9)]);
    
    // Update stats
    setStats(prev => ({
      ...prev,
      totalRatings: prev.totalRatings + 1,
      recentActivity: prev.recentActivity + 1
    }));
  };

  const handleAgentRegistered = (data: any) => {
    console.log('Agent registered:', data);
    // Update stats
    setStats(prev => ({
      ...prev,
      totalAgents: prev.totalAgents + 1,
      recentActivity: prev.recentActivity + 1
    }));
  };

  const handleProposalCreated = (data: any) => {
    console.log('Proposal created:', data);
    // Update stats
    setStats(prev => ({
      ...prev,
      totalProposals: prev.totalProposals + 1,
      recentActivity: prev.recentActivity + 1
    }));
  };

  const handleVoteCast = (data: any) => {
    console.log('Vote cast:', data);
    // Update proposal in active proposals list
    setActiveProposals(prev =>
      prev.map(proposal =>
        proposal.id === data.proposalId
          ? {
              ...proposal,
              votesFor: data.support ? proposal.votesFor + 1 : proposal.votesFor,
              votesAgainst: !data.support ? proposal.votesAgainst + 1 : proposal.votesAgainst
            }
          : proposal
      )
    );
    
    // Update stats
    setStats(prev => ({
      ...prev,
      recentActivity: prev.recentActivity + 1
    }));
  };

  const handleRefresh = () => {
    loadDashboardData();
  };

  if (loading) {
    return (
      <div className="dashboard-loading">
        <div className="loading-spinner"></div>
        <p>Loading dashboard...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="dashboard-error">
        <h2>Error</h2>
        <p>{error}</p>
        <button onClick={handleRefresh} className="retry-button">
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h1>Agent-Karma Dashboard</h1>
        <div className="dashboard-controls">
          <div className={`connection-status ${wsConnected ? 'connected' : 'disconnected'}`}>
            <span className="status-indicator"></span>
            {wsConnected ? 'Live' : 'Offline'}
          </div>
          <button onClick={handleRefresh} className="refresh-button">
            Refresh
          </button>
        </div>
      </div>

      <div className="dashboard-content">
        <div className="dashboard-row">
          <StatsCards stats={stats} />
        </div>

        <div className="dashboard-row">
          <div className="dashboard-col-2">
            <KarmaChart />
          </div>
          <div className="dashboard-col-1">
            <RecentActivity 
              ratings={recentRatings} 
              wsConnected={wsConnected}
            />
          </div>
        </div>

        <div className="dashboard-row">
          <div className="dashboard-col-2">
            <Leaderboard 
              agents={leaderboard}
              onRefresh={handleRefresh}
            />
          </div>
          <div className="dashboard-col-1">
            <ProposalsList 
              proposals={activeProposals}
              onRefresh={handleRefresh}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;

