import { createClient } from 'redis';
import { logger } from '../middleware/errorHandler';

const client = createClient({
  url: process.env.REDIS_URL || 'redis://localhost:6379',
  socket: {
    connectTimeout: 5000,
  },
});

client.on('connect', () => {
  logger.info('Connected to Redis cache');
});

client.on('error', (err) => {
  logger.error('Redis client error', err);
});

client.on('ready', () => {
  logger.info('Redis client ready');
});

client.on('reconnecting', () => {
  logger.info('Redis client reconnecting');
});

export const connectCache = async () => {
  try {
    await client.connect();
    logger.info('Redis cache connected successfully');
  } catch (error) {
    logger.error('Failed to connect to Redis cache', error);
    // Don't exit process, allow app to run without cache
  }
};

export const disconnectCache = async () => {
  try {
    await client.disconnect();
    logger.info('Redis cache disconnected');
  } catch (error) {
    logger.error('Error disconnecting from Redis cache', error);
  }
};

// Cache utility functions
export const cacheService = {
  // Get value from cache
  async get(key: string): Promise<string | null> {
    try {
      if (!client.isReady) return null;
      return await client.get(key);
    } catch (error) {
      logger.error(`Cache get error for key ${key}`, error);
      return null;
    }
  },

  // Set value in cache with TTL
  async set(key: string, value: string, ttlSeconds: number = 300): Promise<boolean> {
    try {
      if (!client.isReady) return false;
      await client.setEx(key, ttlSeconds, value);
      return true;
    } catch (error) {
      logger.error(`Cache set error for key ${key}`, error);
      return false;
    }
  },

  // Delete key from cache
  async del(key: string): Promise<boolean> {
    try {
      if (!client.isReady) return false;
      await client.del(key);
      return true;
    } catch (error) {
      logger.error(`Cache delete error for key ${key}`, error);
      return false;
    }
  },

  // Check if key exists
  async exists(key: string): Promise<boolean> {
    try {
      if (!client.isReady) return false;
      const result = await client.exists(key);
      return result === 1;
    } catch (error) {
      logger.error(`Cache exists error for key ${key}`, error);
      return false;
    }
  },

  // Increment counter
  async incr(key: string): Promise<number | null> {
    try {
      if (!client.isReady) return null;
      return await client.incr(key);
    } catch (error) {
      logger.error(`Cache incr error for key ${key}`, error);
      return null;
    }
  },

  // Set expiration for key
  async expire(key: string, seconds: number): Promise<boolean> {
    try {
      if (!client.isReady) return false;
      await client.expire(key, seconds);
      return true;
    } catch (error) {
      logger.error(`Cache expire error for key ${key}`, error);
      return false;
    }
  },

  // Get multiple keys
  async mget(keys: string[]): Promise<(string | null)[]> {
    try {
      if (!client.isReady) return keys.map(() => null);
      return await client.mGet(keys);
    } catch (error) {
      logger.error(`Cache mget error for keys ${keys.join(', ')}`, error);
      return keys.map(() => null);
    }
  },

  // Set multiple key-value pairs
  async mset(keyValues: Record<string, string>): Promise<boolean> {
    try {
      if (!client.isReady) return false;
      await client.mSet(keyValues);
      return true;
    } catch (error) {
      logger.error('Cache mset error', error);
      return false;
    }
  },

  // Clear all cache
  async flushAll(): Promise<boolean> {
    try {
      if (!client.isReady) return false;
      await client.flushAll();
      return true;
    } catch (error) {
      logger.error('Cache flush error', error);
      return false;
    }
  },

  // Get cache info
  async info(): Promise<string | null> {
    try {
      if (!client.isReady) return null;
      return await client.info();
    } catch (error) {
      logger.error('Cache info error', error);
      return null;
    }
  }
};

// Cache key generators
export const cacheKeys = {
  agentKarma: (address: string) => `agent:karma:${address}`,
  agentDetails: (address: string) => `agent:details:${address}`,
  agentHistory: (address: string, page: number, limit: number) => `agent:history:${address}:${page}:${limit}`,
  agentInteractions: (address: string, page: number, limit: number) => `agent:interactions:${address}:${page}:${limit}`,
  leaderboard: (page: number, limit: number) => `leaderboard:${page}:${limit}`,
  ratings: (address: string, page: number, limit: number) => `ratings:${address}:${page}:${limit}`,
  ratingStats: (address: string) => `rating:stats:${address}`,
  proposals: (page: number, limit: number, status?: string) => `proposals:${page}:${limit}:${status || 'all'}`,
  proposal: (id: string) => `proposal:${id}`,
  proposalVotes: (id: string, page: number, limit: number) => `proposal:votes:${id}:${page}:${limit}`,
  governanceStats: () => 'governance:stats'
};

export default client;

