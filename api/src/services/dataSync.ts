import { query } from './database';
import { cacheService, cacheKeys } from './cache';
import { logger } from '../middleware/errorHandler';
import { Agent, Rating, Interaction, Proposal, Vote, KarmaHistory } from '../types';

export class DataSyncService {
  // Sync agent data between blockchain and database
  async syncAgent(agentData: Agent): Promise<void> {
    try {
      // Update database
      await query(`
        INSERT INTO agents (address, name, description, capabilities, ipfs_hash, karma, registration_date, last_update, interaction_count)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (address) 
        DO UPDATE SET 
          name = EXCLUDED.name,
          description = EXCLUDED.description,
          capabilities = EXCLUDED.capabilities,
          ipfs_hash = EXCLUDED.ipfs_hash,
          karma = EXCLUDED.karma,
          last_update = EXCLUDED.last_update,
          interaction_count = EXCLUDED.interaction_count
      `, [
        agentData.address,
        agentData.metadata.name,
        agentData.metadata.description,
        agentData.metadata.capabilities,
        agentData.metadata.ipfsHash,
        agentData.karma,
        agentData.registrationDate,
        agentData.lastUpdate,
        agentData.interactionCount
      ]);

      // Update cache
      await cacheService.set(
        cacheKeys.agentDetails(agentData.address),
        JSON.stringify(agentData),
        300 // 5 minutes TTL
      );

      await cacheService.set(
        cacheKeys.agentKarma(agentData.address),
        agentData.karma.toString(),
        60 // 1 minute TTL for karma scores
      );

      logger.info(`Synced agent data for ${agentData.address}`);
    } catch (error) {
      logger.error(`Failed to sync agent data for ${agentData.address}`, error);
      throw error;
    }
  }

  // Sync rating data
  async syncRating(ratingData: Rating): Promise<void> {
    try {
      // Update database
      await query(`
        INSERT INTO ratings (rater_address, rated_address, score, interaction_hash, context, timestamp, block_height)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (interaction_hash) DO NOTHING
      `, [
        ratingData.raterAddress,
        ratingData.ratedAddress,
        ratingData.score,
        ratingData.interactionHash,
        ratingData.context,
        ratingData.timestamp,
        ratingData.blockHeight
      ]);

      // Invalidate related cache entries
      await this.invalidateRatingCache(ratingData.ratedAddress);
      await this.invalidateRatingCache(ratingData.raterAddress);

      logger.info(`Synced rating data: ${ratingData.raterAddress} -> ${ratingData.ratedAddress}`);
    } catch (error) {
      logger.error(`Failed to sync rating data`, error);
      throw error;
    }
  }

  // Sync interaction data
  async syncInteraction(interactionData: Interaction): Promise<void> {
    try {
      // Update database
      await query(`
        INSERT INTO interactions (hash, participants, type, timestamp, block_height, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (hash) DO NOTHING
      `, [
        interactionData.hash,
        interactionData.participants,
        interactionData.type,
        interactionData.timestamp,
        interactionData.blockHeight,
        JSON.stringify(interactionData.metadata)
      ]);

      // Invalidate interaction cache for all participants
      for (const participant of interactionData.participants) {
        await this.invalidateInteractionCache(participant);
      }

      logger.info(`Synced interaction data: ${interactionData.hash}`);
    } catch (error) {
      logger.error(`Failed to sync interaction data`, error);
      throw error;
    }
  }

  // Sync karma history
  async syncKarmaHistory(historyData: KarmaHistory): Promise<void> {
    try {
      // Update database
      await query(`
        INSERT INTO karma_history (agent_address, score, timestamp, block_height, reason)
        VALUES ($1, $2, $3, $4, $5)
      `, [
        historyData.address,
        historyData.score,
        historyData.timestamp,
        historyData.blockHeight,
        historyData.reason
      ]);

      // Invalidate karma history cache
      await this.invalidateKarmaHistoryCache(historyData.address);

      logger.info(`Synced karma history for ${historyData.address}`);
    } catch (error) {
      logger.error(`Failed to sync karma history`, error);
      throw error;
    }
  }

  // Sync proposal data
  async syncProposal(proposalData: Proposal): Promise<void> {
    try {
      // Update database
      await query(`
        INSERT INTO proposals (id, title, description, proposer, status, votes_for, votes_against, quorum, deadline, created_at, executed_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (id) 
        DO UPDATE SET 
          status = EXCLUDED.status,
          votes_for = EXCLUDED.votes_for,
          votes_against = EXCLUDED.votes_against,
          executed_at = EXCLUDED.executed_at
      `, [
        proposalData.id,
        proposalData.title,
        proposalData.description,
        proposalData.proposer,
        proposalData.status,
        proposalData.votesFor,
        proposalData.votesAgainst,
        proposalData.quorum,
        proposalData.deadline,
        proposalData.createdAt,
        proposalData.executedAt
      ]);

      // Update cache
      await cacheService.set(
        cacheKeys.proposal(proposalData.id),
        JSON.stringify(proposalData),
        300 // 5 minutes TTL
      );

      // Invalidate proposals list cache
      await this.invalidateProposalsCache();

      logger.info(`Synced proposal data: ${proposalData.id}`);
    } catch (error) {
      logger.error(`Failed to sync proposal data`, error);
      throw error;
    }
  }

  // Sync vote data
  async syncVote(voteData: Vote): Promise<void> {
    try {
      // Update database
      await query(`
        INSERT INTO votes (proposal_id, voter, support, voting_power, timestamp)
        VALUES ($1, $2, $3, $4, $5)
      `, [
        voteData.proposalId,
        voteData.voter,
        voteData.support,
        voteData.votingPower,
        voteData.timestamp
      ]);

      // Invalidate proposal votes cache
      await this.invalidateProposalVotesCache(voteData.proposalId);

      logger.info(`Synced vote data: ${voteData.voter} -> ${voteData.proposalId}`);
    } catch (error) {
      logger.error(`Failed to sync vote data`, error);
      throw error;
    }
  }

  // Cache invalidation methods
  private async invalidateRatingCache(address: string): Promise<void> {
    const patterns = [
      `ratings:${address}:*`,
      `rating:stats:${address}`,
      'leaderboard:*'
    ];

    for (const pattern of patterns) {
      // Note: In production, you'd want to use Redis SCAN with pattern matching
      // For now, we'll invalidate specific known keys
      if (pattern.includes('*')) {
        // Invalidate common pagination combinations
        for (let page = 1; page <= 10; page++) {
          for (const limit of [10, 20, 50]) {
            const key = pattern.replace('*', `${page}:${limit}`);
            await cacheService.del(key);
          }
        }
      } else {
        await cacheService.del(pattern);
      }
    }
  }

  private async invalidateInteractionCache(address: string): Promise<void> {
    // Invalidate interaction cache for agent
    for (let page = 1; page <= 10; page++) {
      for (const limit of [10, 20, 50]) {
        await cacheService.del(cacheKeys.agentInteractions(address, page, limit));
      }
    }
  }

  private async invalidateKarmaHistoryCache(address: string): Promise<void> {
    // Invalidate karma history cache for agent
    for (let page = 1; page <= 10; page++) {
      for (const limit of [10, 20, 50]) {
        await cacheService.del(cacheKeys.agentHistory(address, page, limit));
      }
    }
  }

  private async invalidateProposalsCache(): Promise<void> {
    // Invalidate proposals list cache
    for (let page = 1; page <= 10; page++) {
      for (const limit of [10, 20, 50]) {
        await cacheService.del(cacheKeys.proposals(page, limit));
        await cacheService.del(cacheKeys.proposals(page, limit, 'active'));
        await cacheService.del(cacheKeys.proposals(page, limit, 'passed'));
        await cacheService.del(cacheKeys.proposals(page, limit, 'rejected'));
      }
    }
    await cacheService.del(cacheKeys.governanceStats());
  }

  private async invalidateProposalVotesCache(proposalId: string): Promise<void> {
    // Invalidate proposal votes cache
    for (let page = 1; page <= 10; page++) {
      for (const limit of [10, 20, 50]) {
        await cacheService.del(cacheKeys.proposalVotes(proposalId, page, limit));
      }
    }
  }

  // Bulk sync operations
  async bulkSyncAgents(agents: Agent[]): Promise<void> {
    logger.info(`Starting bulk sync of ${agents.length} agents`);
    
    for (const agent of agents) {
      try {
        await this.syncAgent(agent);
      } catch (error) {
        logger.error(`Failed to sync agent ${agent.address} in bulk operation`, error);
        // Continue with other agents
      }
    }
    
    logger.info(`Completed bulk sync of agents`);
  }

  async bulkSyncRatings(ratings: Rating[]): Promise<void> {
    logger.info(`Starting bulk sync of ${ratings.length} ratings`);
    
    for (const rating of ratings) {
      try {
        await this.syncRating(rating);
      } catch (error) {
        logger.error(`Failed to sync rating in bulk operation`, error);
        // Continue with other ratings
      }
    }
    
    logger.info(`Completed bulk sync of ratings`);
  }

  // Health check for data sync
  async healthCheck(): Promise<{ database: boolean; cache: boolean }> {
    let databaseHealthy = false;
    let cacheHealthy = false;

    try {
      await query('SELECT 1');
      databaseHealthy = true;
    } catch (error) {
      logger.error('Database health check failed', error);
    }

    try {
      await cacheService.set('health_check', 'ok', 10);
      const result = await cacheService.get('health_check');
      cacheHealthy = result === 'ok';
      await cacheService.del('health_check');
    } catch (error) {
      logger.error('Cache health check failed', error);
    }

    return { database: databaseHealthy, cache: cacheHealthy };
  }
}

export const dataSyncService = new DataSyncService();

