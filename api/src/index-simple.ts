// Simplified Agent-Karma API Server for testing
// Version without database and cache dependencies

import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import { createServer } from 'http';

// Import middleware
import { errorHandler, notFoundHandler, logger } from './middleware/errorHandler';

// Import WebSocket service
import WebSocketService from './services/websocket';

const app = express();
const server = createServer(app);
const PORT = process.env.API_PORT || 3000;

// Initialize WebSocket service
const wsService = new WebSocketService(server);

// Middleware to inject WebSocket service into requests
app.use((req, res, next) => {
  (req as any).wsService = wsService;
  next();
});

// Security middleware
app.use(helmet({
  crossOriginEmbedderPolicy: false,
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
    },
  },
}));

// CORS configuration
app.use(cors({
  origin: "*",
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization'],
  credentials: true
}));

// Body parsing middleware
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Rate limiting
const limiter = rateLimit({
  windowMs: 900000, // 15 minutes
  max: 100,
  message: {
    success: false,
    error: 'Too many requests from this IP, please try again later.',
    timestamp: new Date().toISOString()
  },
  standardHeaders: true,
  legacyHeaders: false,
});
app.use('/api/', limiter);

// Request logging middleware
app.use((req, res, next) => {
  logger.info(`${req.method} ${req.url} - ${req.ip}`);
  next();
});

// Health check endpoint
app.get('/health', (req, res) => {
  const wsStats = wsService.getStats();
  res.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    services: {
      websocket: {
        enabled: true,
        connections: wsStats.totalConnections
      },
      database: {
        healthy: false // Disabled for testing
      },
      cache: {
        healthy: false // Disabled for testing
      }
    }
  });
});

// Mock API endpoints for testing
app.get('/api/agents', (req, res) => {
  const mockAgents = [
    {
      address: 'sei1agent1example',
      karma: 1250,
      rank: 1,
      metadata: {
        name: 'AI Assistant Alpha',
        description: 'Advanced reasoning and problem-solving agent'
      }
    },
    {
      address: 'sei1agent2example',
      karma: 980,
      rank: 2,
      metadata: {
        name: 'Data Analyst Bot',
        description: 'Specialized in data analysis and visualization'
      }
    },
    {
      address: 'sei1agent3example',
      karma: 750,
      rank: 3,
      metadata: {
        name: 'Creative Writer AI',
        description: 'Content creation and creative writing specialist'
      }
    }
  ];

  res.json({
    success: true,
    data: mockAgents,
    pagination: {
      page: 1,
      limit: 20,
      total: 3,
      totalPages: 1
    },
    timestamp: new Date().toISOString()
  });
});

app.get('/api/ratings', (req, res) => {
  const mockRatings = [
    {
      id: 'rating1',
      raterAddress: 'sei1rater1example',
      ratedAddress: 'sei1agent1example',
      score: 9,
      interactionHash: '0xhash1',
      context: 'Excellent problem-solving capabilities',
      timestamp: new Date(Date.now() - 300000).toISOString(),
      blockHeight: 12345
    },
    {
      id: 'rating2',
      raterAddress: 'sei1rater2example',
      ratedAddress: 'sei1agent2example',
      score: 8,
      interactionHash: '0xhash2',
      context: 'Great data analysis but could improve speed',
      timestamp: new Date(Date.now() - 600000).toISOString(),
      blockHeight: 12344
    }
  ];

  res.json({
    success: true,
    data: mockRatings,
    pagination: {
      page: 1,
      limit: 20,
      total: 2,
      totalPages: 1
    },
    timestamp: new Date().toISOString()
  });
});

app.get('/api/governance/proposals', (req, res) => {
  const mockProposals = [
    {
      id: 'proposal1',
      title: 'Increase minimum karma requirement for voting',
      description: 'Proposal to increase the minimum karma requirement from 50 to 100 for participating in governance votes.',
      proposer: 'sei1proposer1example',
      status: 'active',
      votesFor: 15,
      votesAgainst: 3,
      quorum: 20,
      deadline: new Date(Date.now() + 86400000 * 7).toISOString(), // 7 days from now
      createdAt: new Date(Date.now() - 86400000).toISOString(), // 1 day ago
    },
    {
      id: 'proposal2',
      title: 'Add new agent capability categories',
      description: 'Proposal to add new standardized capability categories for better agent classification.',
      proposer: 'sei1proposer2example',
      status: 'active',
      votesFor: 8,
      votesAgainst: 12,
      quorum: 25,
      deadline: new Date(Date.now() + 86400000 * 3).toISOString(), // 3 days from now
      createdAt: new Date(Date.now() - 86400000 * 2).toISOString(), // 2 days ago
    }
  ];

  res.json({
    success: true,
    data: mockProposals,
    pagination: {
      page: 1,
      limit: 20,
      total: 2,
      totalPages: 1
    },
    timestamp: new Date().toISOString()
  });
});

app.get('/api/governance/stats', (req, res) => {
  res.json({
    success: true,
    data: {
      totalProposals: 15,
      activeProposals: 2,
      passedProposals: 8,
      rejectedProposals: 5,
      totalVotes: 234,
      averageParticipation: 65
    },
    timestamp: new Date().toISOString()
  });
});

// API documentation endpoint
app.get('/api', (req, res) => {
  res.json({
    name: 'Agent-Karma API (Test Mode)',
    version: '1.0.0',
    description: 'Simplified REST API for Agent-Karma dashboard testing',
    mode: 'testing',
    features: {
      database: 'Mock data (disabled)',
      cache: 'Disabled for testing',
      websocket: 'Real-time updates with Socket.io',
      security: 'Basic rate limiting and CORS'
    },
    endpoints: {
      'GET /api/agents': 'Get mock agent leaderboard',
      'GET /api/ratings': 'Get mock recent ratings',
      'GET /api/governance/proposals': 'Get mock proposals',
      'GET /api/governance/stats': 'Get mock governance stats'
    },
    websocket: {
      url: `ws://localhost:${PORT}`,
      events: [
        'karma_updated',
        'rating_submitted', 
        'agent_registered',
        'proposal_created',
        'vote_cast'
      ]
    },
    timestamp: new Date().toISOString()
  });
});

// Error handling middleware (must be last)
app.use(notFoundHandler);
app.use(errorHandler);

// Start server
server.listen(Number(PORT), '0.0.0.0', () => {
  logger.info(`Agent-Karma API (Test Mode) running on port ${PORT}`);
  logger.info(`WebSocket server enabled`);
  logger.info(`Health check available at http://localhost:${PORT}/health`);
  logger.info(`API documentation available at http://localhost:${PORT}/api`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down gracefully');
  server.close(() => {
    logger.info('Process terminated');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  logger.info('SIGINT received, shutting down gracefully');
  server.close(() => {
    logger.info('Process terminated');
    process.exit(0);
  });
});

export default app;

