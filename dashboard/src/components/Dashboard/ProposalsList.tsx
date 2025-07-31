import React from 'react';
import { Proposal } from '../../types';

interface ProposalsListProps {
  proposals: Proposal[];
  onRefresh: () => void;
}

const ProposalsList: React.FC<ProposalsListProps> = ({ proposals, onRefresh }) => {
  const formatAddress = (address: string): string => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));

    if (diffDays > 0) return `${diffDays}d ago`;
    if (diffHours > 0) return `${diffHours}h ago`;
    return 'Recently';
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'active': return '#3b82f6';
      case 'passed': return '#059669';
      case 'rejected': return '#dc2626';
      case 'executed': return '#7c3aed';
      default: return '#6b7280';
    }
  };

  const getStatusEmoji = (status: string): string => {
    switch (status) {
      case 'active': return '🗳️';
      case 'passed': return '✅';
      case 'rejected': return '❌';
      case 'executed': return '⚡';
      default: return '📋';
    }
  };

  const calculateProgress = (proposal: Proposal): number => {
    const totalVotes = proposal.votesFor + proposal.votesAgainst;
    if (totalVotes === 0) return 0;
    return Math.round((proposal.votesFor / totalVotes) * 100);
  };

  const getTimeRemaining = (deadline: string): string => {
    const deadlineDate = new Date(deadline);
    const now = new Date();
    const diffMs = deadlineDate.getTime() - now.getTime();
    
    if (diffMs <= 0) return 'Expired';
    
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    
    if (diffDays > 0) return `${diffDays}d ${diffHours}h left`;
    return `${diffHours}h left`;
  };

  return (
    <div className="dashboard-card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>🗳️ Active Proposals</h2>
        <button onClick={onRefresh} className="refresh-button" style={{ fontSize: '0.8rem', padding: '6px 12px' }}>
          Refresh
        </button>
      </div>

      <div className="proposals-list">
        {proposals.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '40px', color: '#6b7280' }}>
            <div style={{ fontSize: '3rem', marginBottom: '10px' }}>📋</div>
            <p>No active proposals</p>
            <button onClick={onRefresh} style={{ marginTop: '10px', padding: '8px 16px', border: '1px solid #d1d5db', borderRadius: '6px', background: 'white', cursor: 'pointer' }}>
              Load Proposals
            </button>
          </div>
        ) : (
          proposals.map((proposal, index) => {
            const progress = calculateProgress(proposal);
            const timeRemaining = getTimeRemaining(proposal.deadline);
            
            return (
              <div
                key={proposal.id}
                className="proposal-item"
                style={{
                  padding: '16px',
                  marginBottom: '12px',
                  background: '#f9fafb',
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
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '12px' }}>
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', alignItems: 'center', marginBottom: '6px' }}>
                      <span style={{ fontSize: '1.2rem', marginRight: '8px' }}>
                        {getStatusEmoji(proposal.status)}
                      </span>
                      <span 
                        style={{ 
                          fontSize: '0.7rem', 
                          padding: '2px 6px', 
                          borderRadius: '10px', 
                          background: getStatusColor(proposal.status),
                          color: 'white',
                          fontWeight: '500'
                        }}
                      >
                        {proposal.status.toUpperCase()}
                      </span>
                    </div>
                    
                    <h4 style={{ 
                      margin: '0 0 6px 0', 
                      fontSize: '0.9rem', 
                      fontWeight: '600', 
                      color: '#1f2937',
                      lineHeight: '1.3'
                    }}>
                      {proposal.title.length > 50 ? `${proposal.title.substring(0, 50)}...` : proposal.title}
                    </h4>
                    
                    <p style={{ 
                      margin: '0 0 8px 0', 
                      fontSize: '0.75rem', 
                      color: '#6b7280',
                      lineHeight: '1.4'
                    }}>
                      {proposal.description.length > 80 ? `${proposal.description.substring(0, 80)}...` : proposal.description}
                    </p>
                    
                    <div style={{ fontSize: '0.7rem', color: '#9ca3af' }}>
                      By {formatAddress(proposal.proposer)} • {formatTime(proposal.createdAt)}
                    </div>
                  </div>
                </div>

                {/* Voting Progress */}
                <div style={{ marginBottom: '10px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                    <span style={{ fontSize: '0.7rem', color: '#059669', fontWeight: '500' }}>
                      For: {proposal.votesFor}
                    </span>
                    <span style={{ fontSize: '0.7rem', color: '#dc2626', fontWeight: '500' }}>
                      Against: {proposal.votesAgainst}
                    </span>
                  </div>
                  
                  <div style={{ 
                    width: '100%', 
                    height: '6px', 
                    background: '#fee2e2', 
                    borderRadius: '3px',
                    overflow: 'hidden'
                  }}>
                    <div 
                      style={{ 
                        width: `${progress}%`, 
                        height: '100%', 
                        background: 'linear-gradient(90deg, #059669 0%, #34d399 100%)',
                        transition: 'width 0.3s ease'
                      }}
                    />
                  </div>
                  
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px' }}>
                    <span style={{ fontSize: '0.65rem', color: '#6b7280' }}>
                      {progress}% approval
                    </span>
                    <span style={{ fontSize: '0.65rem', color: '#6b7280' }}>
                      Quorum: {proposal.quorum}
                    </span>
                  </div>
                </div>

                {/* Time Remaining */}
                {proposal.status === 'active' && (
                  <div style={{ 
                    padding: '6px 8px', 
                    background: timeRemaining === 'Expired' ? '#fee2e2' : '#eff6ff',
                    borderRadius: '4px',
                    textAlign: 'center'
                  }}>
                    <span style={{ 
                      fontSize: '0.7rem', 
                      color: timeRemaining === 'Expired' ? '#dc2626' : '#3b82f6',
                      fontWeight: '500'
                    }}>
                      ⏰ {timeRemaining}
                    </span>
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>

      {proposals.length > 0 && (
        <div style={{ marginTop: '15px', textAlign: 'center' }}>
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
            View All Proposals
          </button>
        </div>
      )}
    </div>
  );
};

export default ProposalsList;

