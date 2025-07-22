// Agent-Karma API Server
// Main entry point for the REST API and WebSocket services

import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';

const app = express();
const PORT = process.env.API_PORT || 3000;

// Middleware
app.use(helmet());
app.use(cors());
app.use(express.json());

// Rate limiting
const limiter = rateLimit({
  windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '900000'), // 15 minutes
  max: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100'), // limit each IP to 100 requests per windowMs
});
app.use(limiter);

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// TODO: Add route handlers
// app.use('/api/agents', agentRoutes);
// app.use('/api/karma', karmaRoutes);
// app.use('/api/ratings', ratingRoutes);
// app.use('/api/governance', governanceRoutes);

app.listen(PORT, () => {
  console.log(`Agent-Karma API server running on port ${PORT}`);
});

export default app;