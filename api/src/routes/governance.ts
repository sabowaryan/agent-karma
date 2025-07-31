import { Router, Request, Response } from 'express';
import { validate, validateQuery, schemas } from '../middleware/validation';
import { authenticateToken, requireMinimumKarma } from '../middleware/auth';
import { asyncHandler, ApiError } from '../middleware/errorHandler';
import { Proposal, Vote, ApiResponse, PaginatedResponse } from '../types';

const router = Router();

// Create a new proposal
router.post('/proposals',
  authenticateToken,
  requireMinimumKarma(100), // Minimum 100 karma to create proposals
  validate(schemas.createProposal),
  asyncHandler(async (req: Request, res: Response) => {
    const { title, description } = req.body;
    const proposer = req.user!.address;

    try {
      // TODO: Integrate with Agent-Karma SDK
      // const result = await agentKarmaSDK.createProposal({
      //   title,
      //   description,
      //   proposer
      // });

      // Mock response for now
      const proposal: Proposal = {
        id: `proposal_${Date.now()}`,
        title,
        description,
        proposer,
        status: 'active',
        votesFor: 0,
        votesAgainst: 0,
        quorum: 1000, // Minimum votes needed
        deadline: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(), // 7 days from now
        createdAt: new Date().toISOString()
      };

      const response: ApiResponse<Proposal> = {
        success: true,
        data: proposal,
        timestamp: new Date().toISOString()
      };

      res.status(201).json(response);
    } catch (error) {
      throw new ApiError('Failed to create proposal', 500);
    }
  })
);

// Get all proposals
router.get('/proposals',
  validateQuery(schemas.pagination),
  asyncHandler(async (req: Request, res: Response) => {
    const { page, limit } = req.query as any;
    const { status } = req.query;

    try {
      // TODO: Integrate with Agent-Karma SDK
      // const proposals = await agentKarmaSDK.getProposals(page, limit, status);

      // Mock response for now
      const mockProposals = Array.from({ length: limit }, (_, i) => ({
        id: `proposal_${i}`,
        title: `Proposal ${i + 1}: Improve Agent Karma System`,
        description: `This is a detailed description for proposal ${i + 1}`,
        proposer: `sei1${Math.random().toString(36).substring(2, 15)}`,
        status: ['active', 'passed', 'rejected'][Math.floor(Math.random() * 3)] as any,
        votesFor: Math.floor(Math.random() * 2000),
        votesAgainst: Math.floor(Math.random() * 1000),
        quorum: 1000,
        deadline: new Date(Date.now() + Math.random() * 7 * 24 * 60 * 60 * 1000).toISOString(),
        createdAt: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString()
      }));

      const response: PaginatedResponse<Proposal> = {
        success: true,
        data: mockProposals,
        pagination: {
          page,
          limit,
          total: 200,
          totalPages: Math.ceil(200 / limit)
        },
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      throw new ApiError('Failed to get proposals', 500);
    }
  })
);

// Get a specific proposal
router.get('/proposals/:proposalId',
  asyncHandler(async (req: Request, res: Response) => {
    const { proposalId } = req.params;

    try {
      // TODO: Integrate with Agent-Karma SDK
      // const proposal = await agentKarmaSDK.getProposal(proposalId);

      // Mock response for now
      const proposal: Proposal = {
        id: proposalId,
        title: 'Improve Agent Karma Algorithm',
        description: 'This proposal suggests improvements to the karma calculation algorithm to better reflect agent performance and reduce gaming possibilities.',
        proposer: `sei1${Math.random().toString(36).substring(2, 15)}`,
        status: 'active',
        votesFor: 1250,
        votesAgainst: 340,
        quorum: 1000,
        deadline: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString(),
        createdAt: new Date(Date.now() - 4 * 24 * 60 * 60 * 1000).toISOString()
      };

      const response: ApiResponse<Proposal> = {
        success: true,
        data: proposal,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      throw new ApiError('Proposal not found', 404);
    }
  })
);

// Vote on a proposal
router.post('/proposals/:proposalId/vote',
  authenticateToken,
  requireMinimumKarma(50), // Minimum 50 karma to vote
  validate(schemas.vote),
  asyncHandler(async (req: Request, res: Response) => {
    const { proposalId } = req.params;
    const { support } = req.body;
    const voter = req.user!.address;

    try {
      // TODO: Integrate with Agent-Karma SDK
      // const result = await agentKarmaSDK.vote({
      //   proposalId,
      //   voter,
      //   support
      // });

      // Mock response for now
      const vote: Vote = {
        proposalId,
        voter,
        support,
        votingPower: Math.floor(Math.sqrt(Math.random() * 10000)), // Square root of karma
        timestamp: new Date().toISOString()
      };

      const response: ApiResponse<Vote> = {
        success: true,
        data: vote,
        timestamp: new Date().toISOString()
      };

      res.status(201).json(response);
    } catch (error) {
      throw new ApiError('Failed to submit vote', 500);
    }
  })
);

// Get votes for a proposal
router.get('/proposals/:proposalId/votes',
  validateQuery(schemas.pagination),
  asyncHandler(async (req: Request, res: Response) => {
    const { proposalId } = req.params;
    const { page, limit } = req.query as any;

    try {
      // TODO: Integrate with Agent-Karma SDK
      // const votes = await agentKarmaSDK.getProposalVotes(proposalId, page, limit);

      // Mock response for now
      const mockVotes = Array.from({ length: limit }, (_, i) => ({
        proposalId,
        voter: `sei1${Math.random().toString(36).substring(2, 15)}`,
        support: Math.random() > 0.3, // 70% support rate
        votingPower: Math.floor(Math.sqrt(Math.random() * 10000)),
        timestamp: new Date(Date.now() - i * 3600000).toISOString()
      }));

      const response: PaginatedResponse<Vote> = {
        success: true,
        data: mockVotes,
        pagination: {
          page,
          limit,
          total: 1590,
          totalPages: Math.ceil(1590 / limit)
        },
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      throw new ApiError('Failed to get proposal votes', 500);
    }
  })
);

// Finalize a proposal (execute if passed)
router.post('/proposals/:proposalId/finalize',
  authenticateToken,
  requireMinimumKarma(200), // Higher karma requirement for finalization
  asyncHandler(async (req: Request, res: Response) => {
    const { proposalId } = req.params;

    try {
      // TODO: Integrate with Agent-Karma SDK
      // const result = await agentKarmaSDK.finalizeProposal(proposalId);

      // Mock response for now
      const proposal: Proposal = {
        id: proposalId,
        title: 'Improve Agent Karma Algorithm',
        description: 'This proposal suggests improvements to the karma calculation algorithm.',
        proposer: `sei1${Math.random().toString(36).substring(2, 15)}`,
        status: 'executed',
        votesFor: 1250,
        votesAgainst: 340,
        quorum: 1000,
        deadline: new Date(Date.now() - 1000).toISOString(), // Past deadline
        createdAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
        executedAt: new Date().toISOString()
      };

      const response: ApiResponse<Proposal> = {
        success: true,
        data: proposal,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      throw new ApiError('Failed to finalize proposal', 500);
    }
  })
);

// Get governance statistics
router.get('/stats',
  asyncHandler(async (req: Request, res: Response) => {
    try {
      // TODO: Integrate with Agent-Karma SDK
      // const stats = await agentKarmaSDK.getGovernanceStats();

      // Mock response for now
      const stats = {
        totalProposals: 156,
        activeProposals: 12,
        passedProposals: 89,
        rejectedProposals: 55,
        totalVotes: 15670,
        averageParticipation: 0.23, // 23% of eligible agents vote on average
        topVoters: Array.from({ length: 10 }, (_, i) => ({
          address: `sei1${Math.random().toString(36).substring(2, 15)}`,
          votesCount: Math.floor(Math.random() * 100) + 50,
          karma: Math.floor(Math.random() * 5000) + 1000
        }))
      };

      const response: ApiResponse<any> = {
        success: true,
        data: stats,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      throw new ApiError('Failed to get governance statistics', 500);
    }
  })
);

export default router;

