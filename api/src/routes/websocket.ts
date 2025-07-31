import { Router, Request, Response } from 'express';
import { authenticateToken } from '../middleware/auth';
import { asyncHandler, ApiError } from '../middleware/errorHandler';
import { ApiResponse } from '../types';
import WebSocketService from '../services/websocket';

const router = Router();

// Get WebSocket connection statistics
router.get('/stats',
  authenticateToken,
  asyncHandler(async (req: Request, res: Response) => {
    try {
      // This will be injected by the main app
      const wsService = (req as any).wsService as WebSocketService;
      
      if (!wsService) {
        throw new ApiError('WebSocket service not available', 503);
      }

      const stats = wsService.getStats();

      const response: ApiResponse<any> = {
        success: true,
        data: stats,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      throw new ApiError('Failed to get WebSocket statistics', 500);
    }
  })
);

// Send test event (for development/testing)
router.post('/test-event',
  authenticateToken,
  asyncHandler(async (req: Request, res: Response) => {
    const { eventType, data } = req.body;
    
    try {
      const wsService = (req as any).wsService as WebSocketService;
      
      if (!wsService) {
        throw new ApiError('WebSocket service not available', 503);
      }

      // Send test event based on type
      switch (eventType) {
        case 'karma_updated':
          wsService.broadcastKarmaUpdate(
            data.agentAddress || 'sei1test',
            data.newKarma || 100,
            data.oldKarma || 50
          );
          break;
        case 'rating_submitted':
          wsService.broadcastRatingSubmitted(
            data.raterAddress || 'sei1rater',
            data.ratedAddress || 'sei1rated',
            data.score || 8,
            data.interactionHash || '0xtest'
          );
          break;
        case 'agent_registered':
          wsService.broadcastAgentRegistered(
            data.agentAddress || 'sei1newagent',
            data.metadata || { name: 'Test Agent' }
          );
          break;
        case 'proposal_created':
          wsService.broadcastProposalCreated(
            data.proposalId || 'proposal_test',
            data.title || 'Test Proposal',
            data.proposer || 'sei1proposer'
          );
          break;
        case 'vote_cast':
          wsService.broadcastVoteCast(
            data.proposalId || 'proposal_test',
            data.voter || 'sei1voter',
            data.support !== undefined ? data.support : true,
            data.votingPower || 10
          );
          break;
        default:
          throw new ApiError('Invalid event type', 400);
      }

      const response: ApiResponse<any> = {
        success: true,
        data: { message: `Test event ${eventType} sent successfully` },
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      throw new ApiError('Failed to send test event', 500);
    }
  })
);

export default router;

