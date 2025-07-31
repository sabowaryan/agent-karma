import React from 'react';
import { DashboardStats } from '../../types';

interface StatsCardsProps {
  stats: DashboardStats;
}

const StatsCards: React.FC<StatsCardsProps> = ({ stats }) => {
  const formatNumber = (num: number): string => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M';
    }
    if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K';
    }
    return num.toString();
  };

  const statsData = [
    {
      label: 'Total Agents',
      value: stats.totalAgents,
      change: '+12%',
      changeType: 'positive' as const,
      icon: '👥'
    },
    {
      label: 'Total Ratings',
      value: stats.totalRatings,
      change: '+8%',
      changeType: 'positive' as const,
      icon: '⭐'
    },
    {
      label: 'Active Proposals',
      value: stats.totalProposals,
      change: '+3%',
      changeType: 'positive' as const,
      icon: '📋'
    },
    {
      label: 'Average Karma',
      value: stats.averageKarma,
      change: '+5%',
      changeType: 'positive' as const,
      icon: '🏆'
    },
    {
      label: 'Active Agents',
      value: stats.activeAgents,
      change: '+15%',
      changeType: 'positive' as const,
      icon: '🔥'
    },
    {
      label: 'Recent Activity',
      value: stats.recentActivity,
      change: 'Live',
      changeType: 'neutral' as const,
      icon: '📈'
    }
  ];

  return (
    <div className="stats-cards">
      {statsData.map((stat, index) => (
        <div key={index} className="stat-card">
          <div className="stat-icon" style={{ fontSize: '2rem', marginBottom: '10px' }}>
            {stat.icon}
          </div>
          <div className="stat-value">
            {formatNumber(stat.value)}
          </div>
          <div className="stat-label">
            {stat.label}
          </div>
          <div className={`stat-change ${stat.changeType}`}>
            {stat.change}
          </div>
        </div>
      ))}
    </div>
  );
};

export default StatsCards;

