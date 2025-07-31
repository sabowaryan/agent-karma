import React, { useState, useEffect } from 'react';
import { ChartData } from '../../types';

const KarmaChart: React.FC = () => {
  const [chartData, setChartData] = useState<ChartData | null>(null);
  const [timeRange, setTimeRange] = useState<'day' | 'week' | 'month'>('week');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    generateMockData();
  }, [timeRange]);

  const generateMockData = () => {
    setLoading(true);
    
    // Simulate API call delay
    setTimeout(() => {
      const labels: string[] = [];
      const karmaData: number[] = [];
      const agentData: number[] = [];
      
      let dataPoints = 7;
      let labelFormat = 'day';
      
      switch (timeRange) {
        case 'day':
          dataPoints = 24;
          labelFormat = 'hour';
          break;
        case 'week':
          dataPoints = 7;
          labelFormat = 'day';
          break;
        case 'month':
          dataPoints = 30;
          labelFormat = 'day';
          break;
      }

      for (let i = dataPoints - 1; i >= 0; i--) {
        const date = new Date();
        
        if (labelFormat === 'hour') {
          date.setHours(date.getHours() - i);
          labels.push(date.getHours().toString().padStart(2, '0') + ':00');
        } else {
          date.setDate(date.getDate() - i);
          labels.push(date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' }));
        }
        
        // Generate realistic karma progression
        const baseKarma = 1000 + Math.sin(i * 0.5) * 200;
        const randomVariation = (Math.random() - 0.5) * 100;
        karmaData.push(Math.round(baseKarma + randomVariation));
        
        // Generate agent count data
        const baseAgents = 50 + Math.floor(i * 2);
        const agentVariation = Math.floor(Math.random() * 10);
        agentData.push(baseAgents + agentVariation);
      }

      setChartData({
        labels,
        datasets: [
          {
            label: 'Average Karma',
            data: karmaData,
            borderColor: '#3b82f6',
            backgroundColor: 'rgba(59, 130, 246, 0.1)',
            borderWidth: 2
          },
          {
            label: 'Active Agents',
            data: agentData,
            borderColor: '#059669',
            backgroundColor: 'rgba(5, 150, 105, 0.1)',
            borderWidth: 2
          }
        ]
      });
      
      setLoading(false);
    }, 500);
  };

  const renderSimpleChart = () => {
    if (!chartData) return null;

    const maxKarma = Math.max(...chartData.datasets[0].data);
    const minKarma = Math.min(...chartData.datasets[0].data);
    const karmaRange = maxKarma - minKarma;

    return (
      <div style={{ height: '200px', position: 'relative', padding: '20px 0' }}>
        <svg width="100%" height="100%" viewBox="0 0 400 160" style={{ overflow: 'visible' }}>
          {/* Grid lines */}
          {[0, 1, 2, 3, 4].map(i => (
            <line
              key={i}
              x1="40"
              y1={20 + i * 30}
              x2="380"
              y2={20 + i * 30}
              stroke="#e5e7eb"
              strokeWidth="1"
            />
          ))}
          
          {/* Y-axis labels */}
          {[0, 1, 2, 3, 4].map(i => {
            const value = maxKarma - (i * karmaRange / 4);
            return (
              <text
                key={i}
                x="35"
                y={25 + i * 30}
                textAnchor="end"
                fontSize="10"
                fill="#6b7280"
              >
                {Math.round(value)}
              </text>
            );
          })}
          
          {/* Karma line */}
          <polyline
            points={chartData.datasets[0].data.map((value, index) => {
              const x = 40 + (index * 340 / (chartData.labels.length - 1));
              const y = 140 - ((value - minKarma) / karmaRange) * 120;
              return `${x},${y}`;
            }).join(' ')}
            fill="none"
            stroke="#3b82f6"
            strokeWidth="2"
          />
          
          {/* Data points */}
          {chartData.datasets[0].data.map((value, index) => {
            const x = 40 + (index * 340 / (chartData.labels.length - 1));
            const y = 140 - ((value - minKarma) / karmaRange) * 120;
            return (
              <circle
                key={index}
                cx={x}
                cy={y}
                r="3"
                fill="#3b82f6"
              />
            );
          })}
          
          {/* X-axis labels */}
          {chartData.labels.map((label, index) => {
            if (index % Math.ceil(chartData.labels.length / 6) === 0) {
              const x = 40 + (index * 340 / (chartData.labels.length - 1));
              return (
                <text
                  key={index}
                  x={x}
                  y="155"
                  textAnchor="middle"
                  fontSize="9"
                  fill="#6b7280"
                >
                  {label}
                </text>
              );
            }
            return null;
          })}
        </svg>
      </div>
    );
  };

  return (
    <div className="dashboard-card">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h2>📊 Karma Trends</h2>
        <div style={{ display: 'flex', gap: '8px' }}>
          {(['day', 'week', 'month'] as const).map(range => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              style={{
                padding: '4px 8px',
                border: '1px solid #d1d5db',
                borderRadius: '4px',
                background: timeRange === range ? '#3b82f6' : 'white',
                color: timeRange === range ? 'white' : '#374151',
                cursor: 'pointer',
                fontSize: '0.7rem',
                fontWeight: '500'
              }}
            >
              {range.charAt(0).toUpperCase() + range.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {loading ? (
        <div style={{ 
          height: '200px', 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          color: '#6b7280'
        }}>
          <div>
            <div style={{ 
              width: '30px', 
              height: '30px', 
              border: '3px solid #e5e7eb',
              borderTop: '3px solid #3b82f6',
              borderRadius: '50%',
              animation: 'spin 1s linear infinite',
              margin: '0 auto 10px'
            }}></div>
            Loading chart data...
          </div>
        </div>
      ) : chartData ? (
        <div>
          {renderSimpleChart()}
          
          <div style={{ 
            display: 'flex', 
            justifyContent: 'center', 
            gap: '20px', 
            marginTop: '15px',
            padding: '10px',
            background: '#f9fafb',
            borderRadius: '6px'
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <div style={{ 
                width: '12px', 
                height: '3px', 
                background: '#3b82f6',
                borderRadius: '2px'
              }}></div>
              <span style={{ fontSize: '0.8rem', color: '#374151' }}>Average Karma</span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <div style={{ 
                width: '12px', 
                height: '3px', 
                background: '#059669',
                borderRadius: '2px'
              }}></div>
              <span style={{ fontSize: '0.8rem', color: '#374151' }}>Active Agents</span>
            </div>
          </div>
        </div>
      ) : (
        <div style={{ 
          height: '200px', 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          color: '#6b7280'
        }}>
          <div style={{ textAlign: 'center' }}>
            <div style={{ fontSize: '2rem', marginBottom: '10px' }}>📈</div>
            <p>No chart data available</p>
            <button 
              onClick={generateMockData}
              style={{ 
                marginTop: '10px', 
                padding: '6px 12px', 
                border: '1px solid #d1d5db', 
                borderRadius: '4px', 
                background: 'white', 
                cursor: 'pointer',
                fontSize: '0.8rem'
              }}
            >
              Load Data
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default KarmaChart;

