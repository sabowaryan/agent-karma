// Agent-Karma API Server
// Main entry point for the REST API and WebSocket services

import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import { createServer } from 'http';
import path from 'path';
import fs from 'fs';

// Import middleware
import { errorHandler, notFoundHandler, logger } from './middleware/errorHandler';

// Import routes
import agentRoutes from './routes/agents';
import ratingRoutes from './routes/ratings';
import governanceRoutes from './routes/governance';
import websocketRoutes from './routes/websocket';

// Import services
import WebSocketService from './services/websocket';
import { connectCache, disconnectCache } from './services/cache';
import { initializeDatabase } from './services/database';
import { dataSyncService } from './services/dataSync';

const app = express();
const server = createServer(app);
const PORT = process.env.API_PORT || 3000;

// Initialize services
let wsService: WebSocketService;

async function initializeServices() {
  try {
    // Initialize database
    await initializeDatabase();
    logger.info('Database initialized successfully');

    // Connect to cache
    await connectCache();
    logger.info('Cache connected successfully');

    // Initialize WebSocket service
    wsService = new WebSocketService(server);
    logger.info('WebSocket service initialized');

    // Health check for data sync
    const health = await dataSyncService.healthCheck();
    logger.info('Data sync health check:', health);

  } catch (error) {
    logger.error('Failed to initialize services', error);
    process.exit(1);
  }
}

// Middleware to inject services into requests
app.use((req, res, next) => {
  (req as any).wsService = wsService;
  (req as any).dataSyncService = dataSyncService;
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
  origin: process.env.CORS_ORIGIN || "*",
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization'],
  credentials: true
}));

// Body parsing middleware
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Rate limiting
const limiter = rateLimit({
  windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '900000'), // 15 minutes
  max: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100'), // limit each IP to 100 requests per windowMs
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
app.get('/health', async (req, res) => {
  try {
    const wsStats = wsService ? wsService.getStats() : { totalConnections: 0 };
    const dataHealth = await dataSyncService.healthCheck();
    
    res.json({ 
      status: 'ok', 
      timestamp: new Date().toISOString(),
      version: '1.0.0',
      services: {
        websocket: {
          enabled: !!wsService,
          connections: wsStats.totalConnections
        },
        database: {
          healthy: dataHealth.database
        },
        cache: {
          healthy: dataHealth.cache
        }
      }
    });
  } catch (error) {
    logger.error('Health check error', error);
    res.status(503).json({
      status: 'error',
      timestamp: new Date().toISOString(),
      error: 'Service health check failed'
    });
  }
});

// API routes
app.use('/api/agents', agentRoutes);
app.use('/api/ratings', ratingRoutes);
app.use('/api/governance', governanceRoutes);
app.use('/api/websocket', websocketRoutes);

// API documentation endpoint
app.get('/api', (req, res) => {
  res.json({
    name: 'Agent-Karma API',
    version: '1.0.0',
    description: 'REST API and WebSocket services for Agent-Karma decentralized reputation system',
    features: {
      database: 'PostgreSQL with connection pooling',
      cache: 'Redis with TTL-based caching',
      websocket: 'Real-time updates with Socket.io',
      security: 'JWT authentication, rate limiting, CORS'
    },
    endpoints: {
      agents: {
        'POST /api/agents/register': 'Register a new agent',
        'GET /api/agents/:address': 'Get agent details',
        'GET /api/agents/:address/karma': 'Get agent karma score',
        'GET /api/agents/:address/karma/history': 'Get agent karma history',
        'GET /api/agents/:address/interactions': 'Get agent interactions',
        'GET /api/agents': 'Get agent leaderboard'
      },
      ratings: {
        'POST /api/ratings': 'Submit a rating',
        'GET /api/ratings/agent/:address': 'Get ratings for an agent',
        'GET /api/ratings/by/:address': 'Get ratings by an agent',
        'GET /api/ratings': 'Get recent ratings',
        'GET /api/ratings/stats/:address': 'Get rating statistics'
      },
      governance: {
        'POST /api/governance/proposals': 'Create a proposal',
        'GET /api/governance/proposals': 'Get all proposals',
        'GET /api/governance/proposals/:id': 'Get specific proposal',
        'POST /api/governance/proposals/:id/vote': 'Vote on proposal',
        'GET /api/governance/proposals/:id/votes': 'Get proposal votes',
        'POST /api/governance/proposals/:id/finalize': 'Finalize proposal',
        'GET /api/governance/stats': 'Get governance statistics'
      },
      websocket: {
        'GET /api/websocket/stats': 'Get WebSocket statistics',
        'POST /api/websocket/test-event': 'Send test WebSocket event'
      }
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

// Create logs directory if it doesn't exist
const logsDir = path.join(__dirname, '../logs');
if (!fs.existsSync(logsDir)) {
  fs.mkdirSync(logsDir, { recursive: true });
}

// Error handling middleware (must be last)
app.use(notFoundHandler);
app.use(errorHandler);

// Start server
async function startServer() {
  try {
    await initializeServices();
    
    server.listen(Number(PORT), '0.0.0.0', () => {
      logger.info(`Agent-Karma API server running on port ${PORT}`);
      logger.info(`WebSocket server enabled`);
      logger.info(`Database and cache services initialized`);
      logger.info(`Health check available at http://localhost:${PORT}/health`);
      logger.info(`API documentation available at http://localhost:${PORT}/api`);
    });
  } catch (error) {
    logger.error('Failed to start server', error);
    process.exit(1);
  }
}

// Graceful shutdown
async function gracefulShutdown() {
  logger.info('Shutting down gracefully...');
  
  try {
    // Close server
    server.close(() => {
      logger.info('HTTP server closed');
    });

    // Disconnect from cache
    await disconnectCache();
    logger.info('Cache disconnected');

    // Note: Database pool will close automatically when process exits
    
    logger.info('Graceful shutdown completed');
    process.exit(0);
  } catch (error) {
    logger.error('Error during graceful shutdown', error);
    process.exit(1);
  }
}

process.on('SIGTERM', gracefulShutdown);
process.on('SIGINT', gracefulShutdown);

// Start the server
startServer();

export default app;

