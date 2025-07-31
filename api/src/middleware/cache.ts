import { Request, Response, NextFunction } from 'express';
import { cacheService } from '../services/cache';
import { logger } from './errorHandler';

// Cache middleware factory
export const cacheMiddleware = (keyGenerator: (req: Request) => string, ttlSeconds: number = 300) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      const cacheKey = keyGenerator(req);
      const cachedData = await cacheService.get(cacheKey);

      if (cachedData) {
        logger.info(`Cache hit for key: ${cacheKey}`);
        return res.json(JSON.parse(cachedData));
      }

      logger.info(`Cache miss for key: ${cacheKey}`);

      // Store original res.json to intercept response
      const originalJson = res.json;
      res.json = function(data: any) {
        // Cache the response data
        cacheService.set(cacheKey, JSON.stringify(data), ttlSeconds)
          .then(() => logger.info(`Cached response for key: ${cacheKey}`))
          .catch(err => logger.error(`Failed to cache response for key: ${cacheKey}`, err));

        // Call original json method
        return originalJson.call(this, data);
      };

      next();
    } catch (error) {
      logger.error('Cache middleware error', error);
      // Continue without cache on error
      next();
    }
  };
};

// Cache key generators for different routes
export const cacheKeyGenerators = {
  agentDetails: (req: Request) => `agent:details:${req.params.address}`,
  agentKarma: (req: Request) => `agent:karma:${req.params.address}`,
  agentHistory: (req: Request) => {
    const { address } = req.params;
    const { page = 1, limit = 20 } = req.query;
    return `agent:history:${address}:${page}:${limit}`;
  },
  agentInteractions: (req: Request) => {
    const { address } = req.params;
    const { page = 1, limit = 20 } = req.query;
    return `agent:interactions:${address}:${page}:${limit}`;
  },
  leaderboard: (req: Request) => {
    const { page = 1, limit = 20 } = req.query;
    return `leaderboard:${page}:${limit}`;
  },
  ratingsForAgent: (req: Request) => {
    const { address } = req.params;
    const { page = 1, limit = 20 } = req.query;
    return `ratings:agent:${address}:${page}:${limit}`;
  },
  ratingsByAgent: (req: Request) => {
    const { address } = req.params;
    const { page = 1, limit = 20 } = req.query;
    return `ratings:by:${address}:${page}:${limit}`;
  },
  recentRatings: (req: Request) => {
    const { page = 1, limit = 20 } = req.query;
    return `ratings:recent:${page}:${limit}`;
  },
  ratingStats: (req: Request) => `rating:stats:${req.params.address}`,
  proposals: (req: Request) => {
    const { page = 1, limit = 20, status } = req.query;
    return `proposals:${page}:${limit}:${status || 'all'}`;
  },
  proposal: (req: Request) => `proposal:${req.params.proposalId}`,
  proposalVotes: (req: Request) => {
    const { proposalId } = req.params;
    const { page = 1, limit = 20 } = req.query;
    return `proposal:votes:${proposalId}:${page}:${limit}`;
  },
  governanceStats: (req: Request) => 'governance:stats'
};

// Specific cache middleware instances
export const cacheAgentDetails = cacheMiddleware(cacheKeyGenerators.agentDetails, 300); // 5 minutes
export const cacheAgentKarma = cacheMiddleware(cacheKeyGenerators.agentKarma, 60); // 1 minute
export const cacheAgentHistory = cacheMiddleware(cacheKeyGenerators.agentHistory, 300); // 5 minutes
export const cacheAgentInteractions = cacheMiddleware(cacheKeyGenerators.agentInteractions, 300); // 5 minutes
export const cacheLeaderboard = cacheMiddleware(cacheKeyGenerators.leaderboard, 120); // 2 minutes
export const cacheRatingsForAgent = cacheMiddleware(cacheKeyGenerators.ratingsForAgent, 300); // 5 minutes
export const cacheRatingsByAgent = cacheMiddleware(cacheKeyGenerators.ratingsByAgent, 300); // 5 minutes
export const cacheRecentRatings = cacheMiddleware(cacheKeyGenerators.recentRatings, 60); // 1 minute
export const cacheRatingStats = cacheMiddleware(cacheKeyGenerators.ratingStats, 300); // 5 minutes
export const cacheProposals = cacheMiddleware(cacheKeyGenerators.proposals, 120); // 2 minutes
export const cacheProposal = cacheMiddleware(cacheKeyGenerators.proposal, 300); // 5 minutes
export const cacheProposalVotes = cacheMiddleware(cacheKeyGenerators.proposalVotes, 120); // 2 minutes
export const cacheGovernanceStats = cacheMiddleware(cacheKeyGenerators.governanceStats, 300); // 5 minutes

// Cache invalidation middleware
export const invalidateCache = (keyPattern: string) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    // Store original res.json to intercept response
    const originalJson = res.json;
    res.json = function(data: any) {
      // Invalidate cache after successful response
      if (res.statusCode >= 200 && res.statusCode < 300) {
        // Simple pattern matching for cache invalidation
        const keysToInvalidate = generateKeysFromPattern(keyPattern, req);
        keysToInvalidate.forEach(key => {
          cacheService.del(key)
            .then(() => logger.info(`Invalidated cache key: ${key}`))
            .catch(err => logger.error(`Failed to invalidate cache key: ${key}`, err));
        });
      }

      // Call original json method
      return originalJson.call(this, data);
    };

    next();
  };
};

// Helper function to generate cache keys from pattern
function generateKeysFromPattern(pattern: string, req: Request): string[] {
  const keys: string[] = [];
  
  // Replace placeholders in pattern with actual values
  let resolvedPattern = pattern;
  
  if (req.params.address) {
    resolvedPattern = resolvedPattern.replace(':address', req.params.address);
  }
  
  if (req.params.proposalId) {
    resolvedPattern = resolvedPattern.replace(':proposalId', req.params.proposalId);
  }

  // For patterns with wildcards, generate common variations
  if (resolvedPattern.includes('*')) {
    // Generate keys for common pagination combinations
    for (let page = 1; page <= 10; page++) {
      for (const limit of [10, 20, 50]) {
        const key = resolvedPattern.replace('*', `${page}:${limit}`);
        keys.push(key);
      }
    }
  } else {
    keys.push(resolvedPattern);
  }

  return keys;
}

// Cache warming functions
export const warmCache = {
  async agentData(address: string) {
    try {
      // Pre-load common agent data into cache
      const keys = [
        `agent:details:${address}`,
        `agent:karma:${address}`,
        `agent:history:${address}:1:20`,
        `agent:interactions:${address}:1:20`,
        `ratings:agent:${address}:1:20`,
        `rating:stats:${address}`
      ];

      logger.info(`Warming cache for agent: ${address}`);
      // Note: In a real implementation, you'd fetch this data from the database
      // and populate the cache
    } catch (error) {
      logger.error(`Failed to warm cache for agent: ${address}`, error);
    }
  },

  async leaderboard() {
    try {
      // Pre-load leaderboard data
      logger.info('Warming leaderboard cache');
      // Note: In a real implementation, you'd fetch leaderboard data
      // and populate the cache
    } catch (error) {
      logger.error('Failed to warm leaderboard cache', error);
    }
  }
};

