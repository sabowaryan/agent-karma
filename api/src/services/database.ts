import { Pool } from 'pg';
import { logger } from '../middleware/errorHandler';

const pool = new Pool({
  user: process.env.DB_USER || 'postgres',
  host: process.env.DB_HOST || 'localhost',
  database: process.env.DB_NAME || 'agentkarma',
  password: process.env.DB_PASSWORD || 'password',
  port: parseInt(process.env.DB_PORT || '5432'),
  max: parseInt(process.env.DB_POOL_MAX || '20'),
  idleTimeoutMillis: parseInt(process.env.DB_POOL_IDLE_TIMEOUT || '30000'),
  connectionTimeoutMillis: parseInt(process.env.DB_POOL_CONNECTION_TIMEOUT || '2000'),
});

pool.on('connect', () => {
  logger.info('Connected to PostgreSQL database');
});

pool.on('error', (err) => {
  logger.error('Unexpected error on idle client', err);
  process.exit(-1); // Exit process if database connection is lost
});

export const query = (text: string, params?: any[]) => pool.query(text, params);

export const initializeDatabase = async () => {
  try {
    await query(`
      CREATE TABLE IF NOT EXISTS agents (
        address VARCHAR(255) PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        description TEXT,
        capabilities TEXT[],
        ipfs_hash VARCHAR(255),
        karma INTEGER NOT NULL DEFAULT 0,
        registration_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        last_update TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        interaction_count INTEGER NOT NULL DEFAULT 0
      );

      CREATE TABLE IF NOT EXISTS karma_history (
        id SERIAL PRIMARY KEY,
        agent_address VARCHAR(255) REFERENCES agents(address) ON DELETE CASCADE,
        score INTEGER NOT NULL,
        timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        block_height INTEGER,
        reason TEXT
      );

      CREATE TABLE IF NOT EXISTS ratings (
        id SERIAL PRIMARY KEY,
        rater_address VARCHAR(255) NOT NULL,
        rated_address VARCHAR(255) REFERENCES agents(address) ON DELETE CASCADE,
        score INTEGER NOT NULL,
        interaction_hash VARCHAR(255) NOT NULL UNIQUE,
        context TEXT,
        timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        block_height INTEGER
      );

      CREATE TABLE IF NOT EXISTS interactions (
        id SERIAL PRIMARY KEY,
        hash VARCHAR(255) NOT NULL UNIQUE,
        participants TEXT[] NOT NULL,
        type VARCHAR(255) NOT NULL,
        timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        block_height INTEGER,
        metadata JSONB
      );

      CREATE TABLE IF NOT EXISTS proposals (
        id VARCHAR(255) PRIMARY KEY,
        title VARCHAR(255) NOT NULL,
        description TEXT NOT NULL,
        proposer VARCHAR(255) NOT NULL,
        status VARCHAR(50) NOT NULL,
        votes_for INTEGER NOT NULL DEFAULT 0,
        votes_against INTEGER NOT NULL DEFAULT 0,
        quorum INTEGER NOT NULL,
        deadline TIMESTAMP WITH TIME ZONE NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        executed_at TIMESTAMP WITH TIME ZONE
      );

      CREATE TABLE IF NOT EXISTS votes (
        id SERIAL PRIMARY KEY,
        proposal_id VARCHAR(255) REFERENCES proposals(id) ON DELETE CASCADE,
        voter VARCHAR(255) NOT NULL,
        support BOOLEAN NOT NULL,
        voting_power INTEGER NOT NULL,
        timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );
    `);
    logger.info('Database schema initialized or already exists');
  } catch (error) {
    logger.error('Error initializing database schema', error);
    process.exit(-1);
  }
};

export default pool;

