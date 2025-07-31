import React, { useState } from 'react';
import { LeaderboardEntry } from '../../types';

interface LeaderboardProps {
  agents: LeaderboardEntry[];
  onRefresh: () => void;
}

const Leaderboard: React.FC<LeaderboardProps> = ({ agents, onRefresh }) => {
  const [sortBy, setSortBy] = useState<'karma' | 'rank'>('rank');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');

  const sortedAgents = [...agents].sort((a, b) => {
    const aValue = sortBy === 'karma' ? a.karma : a.rank;
    const bValue = sortBy === 'karma' ? b.karma : b.rank;
    
    if (sortOrder === 'asc') {
      return aValue - bValue;
    } else {
      return bValue - aValue;
    }
  });

  const handleSort = (field: 'karma' | 'rank') => {
    if (sortBy === field) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(field);
      setSortOrder('asc');
    }
  };

  const formatAddress = (address: string): string => {
    return `${address.slice(0, 8)}...${address.slice(-6)}`;
  };

  const getRankIcon = (rank: number): string => {
    switch (rank) {
      case 1: return '🥇';
      case 2: return '🥈';
      case 3: return '🥉';
      default: return `#${rank}`;
    }
  };

  const getKarmaColor = (karma: number): string => {
    if (karma >= 1000) return '#059669'; // Green
    if (karma >= 500) return '#d97706'; // Orange
    if (karma >= 100) return '#7c3aed'; // Purple
    return '#6b7280'; // Gray
  };

  return (
    <div className="dashboard-card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>🏆 Agent Leaderboard</h2>
        <button onClick={onRefresh} className="refresh-button" style={{ fontSize: '0.8rem', padding: '6px 12px' }}>
          Refresh
        </button>
      </div>

      <div className="leaderboard-controls" style={{ marginBottom: '20px' }}>
        <div style={{ display: 'flex', gap: '10px' }}>
          <button
            onClick={() => handleSort('rank')}
            className={`sort-button ${sortBy === 'rank' ? 'active' : ''}`}
            style={{
              padding: '6px 12px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              background: sortBy === 'rank' ? '#3b82f6' : 'white',
              color: sortBy === 'rank' ? 'white' : '#374151',
              cursor: 'pointer',
              fontSize: '0.8rem'
            }}
          >
            Rank {sortBy === 'rank' && (sortOrder === 'asc' ? '↑' : '↓')}
          </button>
          <button
            onClick={() => handleSort('karma')}
            className={`sort-button ${sortBy === 'karma' ? 'active' : ''}`}
            style={{
              padding: '6px 12px',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              background: sortBy === 'karma' ? '#3b82f6' : 'white',
              color: sortBy === 'karma' ? 'white' : '#374151',
              cursor: 'pointer',
              fontSize: '0.8rem'
            }}
          >
            Karma {sortBy === 'karma' && (sortOrder === 'asc' ? '↑' : '↓')}
          </button>
        </div>
      </div>

      <div className="leaderboard-list">
        {sortedAgents.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '40px', color: '#6b7280' }}>
            <div style={{ fontSize: '3rem', marginBottom: '10px' }}>📊</div>
            <p>No agents found</p>
            <button onClick={onRefresh} style={{ marginTop: '10px', padding: '8px 16px', border: '1px solid #d1d5db', borderRadius: '6px', background: 'white', cursor: 'pointer' }}>
              Load Data
            </button>
          </div>
        ) : (
          sortedAgents.map((agent, index) => (
            <div
              key={agent.address}
              className="leaderboard-item"
              style={{
                display: 'flex',
                alignItems: 'center',
                padding: '12px',
                marginBottom: '8px',
                background: index < 3 ? 'linear-gradient(135deg, #fef3c7 0%, #fde68a 100%)' : '#f9fafb',
                borderRadius: '8px',
                border: '1px solid #e5e7eb',
                transition: 'transform 0.2s ease, box-shadow 0.2s ease'
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.transform = 'translateY(-1px)';
                e.currentTarget.style.boxShadow = '0 4px 12px rgba(0, 0, 0, 0.1)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = 'none';
              }}
            >
              <div style={{ fontSize: '1.2rem', marginRight: '12px', minWidth: '40px' }}>
                {getRankIcon(agent.rank)}
              </div>
              
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: '600', color: '#1f2937', marginBottom: '2px' }}>
                  {agent.metadata.name}
                </div>
                <div style={{ fontSize: '0.8rem', color: '#6b7280', fontFamily: 'monospace' }}>
                  {formatAddress(agent.address)}
                </div>
                {agent.metadata.description && (
                  <div style={{ fontSize: '0.75rem', color: '#9ca3af', marginTop: '2px' }}>
                    {agent.metadata.description.length > 50 
                      ? `${agent.metadata.description.substring(0, 50)}...`
                      : agent.metadata.description
                    }
                  </div>
                )}
              </div>
              
              <div style={{ textAlign: 'right' }}>
                <div 
                  style={{ 
                    fontSize: '1.2rem', 
                    fontWeight: '700', 
                    color: getKarmaColor(agent.karma),
                    marginBottom: '2px'
                  }}
                >
                  {agent.karma.toLocaleString()}
                </div>
                <div style={{ fontSize: '0.7rem', color: '#6b7280' }}>
                  karma
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {sortedAgents.length > 0 && (
        <div style={{ marginTop: '20px', textAlign: 'center' }}>
          <button
            onClick={onRefresh}
            style={{
              padding: '8px 16px',
              border: '1px solid #3b82f6',
              borderRadius: '6px',
              background: 'white',
              color: '#3b82f6',
              cursor: 'pointer',
              fontSize: '0.8rem',
              fontWeight: '500'
            }}
          >
            Load More
          </button>
        </div>
      )}
    </div>
  );
};

export default Leaderboard;

