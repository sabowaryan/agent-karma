import React from 'react';
import { Rating } from '../../types';

interface RecentActivityProps {
  ratings: Rating[];
  wsConnected: boolean;
}

const RecentActivity: React.FC<RecentActivityProps> = ({ ratings, wsConnected }) => {
  const formatAddress = (address: string): string => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${diffDays}d ago`;
  };

  const getScoreColor = (score: number): string => {
    if (score >= 8) return '#059669'; // Green
    if (score >= 6) return '#d97706'; // Orange
    if (score >= 4) return '#7c3aed'; // Purple
    return '#dc2626'; // Red
  };

  const getScoreEmoji = (score: number): string => {
    if (score >= 9) return '🌟';
    if (score >= 7) return '⭐';
    if (score >= 5) return '👍';
    if (score >= 3) return '👎';
    return '💔';
  };

  return (
    <div className="dashboard-card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>📈 Recent Activity</h2>
        <div className={`connection-status ${wsConnected ? 'connected' : 'disconnected'}`} style={{ fontSize: '0.7rem', padding: '4px 8px' }}>
          <span className="status-indicator" style={{ width: '6px', height: '6px' }}></span>
          {wsConnected ? 'Live' : 'Offline'}
        </div>
      </div>

      <div className="activity-list">
        {ratings.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '40px', color: '#6b7280' }}>
            <div style={{ fontSize: '3rem', marginBottom: '10px' }}>🔄</div>
            <p>No recent activity</p>
            <p style={{ fontSize: '0.8rem', marginTop: '5px' }}>
              {wsConnected ? 'Waiting for new ratings...' : 'Connect to see live updates'}
            </p>
          </div>
        ) : (
          ratings.map((rating, index) => (
            <div
              key={rating.id || index}
              className="activity-item"
              style={{
                display: 'flex',
                alignItems: 'center',
                padding: '12px',
                marginBottom: '8px',
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
              <div style={{ fontSize: '1.5rem', marginRight: '12px' }}>
                {getScoreEmoji(rating.score)}
              </div>
              
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: '0.9rem', color: '#1f2937', marginBottom: '4px' }}>
                  <span style={{ fontFamily: 'monospace', background: '#e5e7eb', padding: '2px 4px', borderRadius: '3px' }}>
                    {formatAddress(rating.raterAddress)}
                  </span>
                  <span style={{ margin: '0 8px', color: '#6b7280' }}>rated</span>
                  <span style={{ fontFamily: 'monospace', background: '#e5e7eb', padding: '2px 4px', borderRadius: '3px' }}>
                    {formatAddress(rating.ratedAddress)}
                  </span>
                </div>
                
                {rating.context && (
                  <div style={{ fontSize: '0.75rem', color: '#6b7280', marginBottom: '4px' }}>
                    "{rating.context.length > 60 ? `${rating.context.substring(0, 60)}...` : rating.context}"
                  </div>
                )}
                
                <div style={{ fontSize: '0.7rem', color: '#9ca3af' }}>
                  {formatTime(rating.timestamp)}
                </div>
              </div>
              
              <div style={{ textAlign: 'right' }}>
                <div 
                  style={{ 
                    fontSize: '1.4rem', 
                    fontWeight: '700', 
                    color: getScoreColor(rating.score),
                    marginBottom: '2px'
                  }}
                >
                  {rating.score}
                </div>
                <div style={{ fontSize: '0.6rem', color: '#6b7280' }}>
                  /10
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {wsConnected && ratings.length > 0 && (
        <div style={{ 
          marginTop: '15px', 
          padding: '10px', 
          background: 'linear-gradient(135deg, #ecfdf5 0%, #d1fae5 100%)', 
          borderRadius: '6px',
          textAlign: 'center'
        }}>
          <div style={{ fontSize: '0.8rem', color: '#059669', fontWeight: '500' }}>
            🔴 Live updates enabled
          </div>
          <div style={{ fontSize: '0.7rem', color: '#065f46', marginTop: '2px' }}>
            New ratings will appear automatically
          </div>
        </div>
      )}
    </div>
  );
};

export default RecentActivity;

