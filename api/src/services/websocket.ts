import { Server as SocketIOServer } from 'socket.io';
import { Server as HTTPServer } from 'http';
import jwt from 'jsonwebtoken';
import { WebSocketEvent, AuthToken } from '../types';
import { logger } from '../middleware/errorHandler';

const JWT_SECRET = process.env.JWT_SECRET || 'agent-karma-secret-key';

export class WebSocketService {
  private io: SocketIOServer;
  private connectedClients: Map<string, string> = new Map(); // socketId -> address

  constructor(server: HTTPServer) {
    this.io = new SocketIOServer(server, {
      cors: {
        origin: "*",
        methods: ["GET", "POST"]
      },
      transports: ['websocket', 'polling']
    });

    this.setupMiddleware();
    this.setupEventHandlers();
  }

  private setupMiddleware() {
    // Authentication middleware for WebSocket
    this.io.use((socket, next) => {
      const token = socket.handshake.auth.token || socket.handshake.headers.authorization?.split(' ')[1];
      
      if (!token) {
        // Allow anonymous connections for public data
        socket.data.isAuthenticated = false;
        return next();
      }

      jwt.verify(token, JWT_SECRET, (err: any, decoded: any) => {
        if (err) {
          socket.data.isAuthenticated = false;
          logger.warn(`WebSocket authentication failed: ${err.message}`);
        } else {
          socket.data.user = decoded as AuthToken;
          socket.data.isAuthenticated = true;
          this.connectedClients.set(socket.id, (decoded as AuthToken).address);
        }
        next();
      });
    });
  }

  private setupEventHandlers() {
    this.io.on('connection', (socket) => {
      const userAddress = socket.data.user?.address || 'anonymous';
      logger.info(`WebSocket client connected: ${socket.id} (${userAddress})`);

      // Join general room for public updates
      socket.join('public');

      // Join user-specific room if authenticated
      if (socket.data.isAuthenticated) {
        socket.join(`user:${userAddress}`);
      }

      // Handle subscription to specific events
      socket.on('subscribe', (eventTypes: string[]) => {
        eventTypes.forEach(eventType => {
          if (this.isValidEventType(eventType)) {
            socket.join(`events:${eventType}`);
            logger.info(`Client ${socket.id} subscribed to ${eventType}`);
          }
        });
      });

      // Handle unsubscription from events
      socket.on('unsubscribe', (eventTypes: string[]) => {
        eventTypes.forEach(eventType => {
          socket.leave(`events:${eventType}`);
          logger.info(`Client ${socket.id} unsubscribed from ${eventType}`);
        });
      });

      // Handle agent-specific subscriptions
      socket.on('subscribe_agent', (agentAddress: string) => {
        if (this.isValidAddress(agentAddress)) {
          socket.join(`agent:${agentAddress}`);
          logger.info(`Client ${socket.id} subscribed to agent ${agentAddress}`);
        }
      });

      // Handle proposal-specific subscriptions
      socket.on('subscribe_proposal', (proposalId: string) => {
        socket.join(`proposal:${proposalId}`);
        logger.info(`Client ${socket.id} subscribed to proposal ${proposalId}`);
      });

      // Handle ping/pong for connection health
      socket.on('ping', () => {
        socket.emit('pong', { timestamp: new Date().toISOString() });
      });

      // Handle disconnection
      socket.on('disconnect', (reason) => {
        this.connectedClients.delete(socket.id);
        logger.info(`WebSocket client disconnected: ${socket.id} (${reason})`);
      });

      // Send welcome message
      socket.emit('connected', {
        message: 'Connected to Agent-Karma WebSocket',
        authenticated: socket.data.isAuthenticated,
        timestamp: new Date().toISOString()
      });
    });
  }

  // Broadcast karma update to relevant subscribers
  public broadcastKarmaUpdate(agentAddress: string, newKarma: number, oldKarma: number) {
    const event: WebSocketEvent = {
      type: 'karma_updated',
      data: {
        agentAddress,
        newKarma,
        oldKarma,
        change: newKarma - oldKarma
      },
      timestamp: new Date().toISOString()
    };

    // Send to public room
    this.io.to('public').emit('karma_updated', event);
    
    // Send to agent-specific subscribers
    this.io.to(`agent:${agentAddress}`).emit('karma_updated', event);
    
    // Send to karma event subscribers
    this.io.to('events:karma_updated').emit('karma_updated', event);

    logger.info(`Broadcasted karma update for ${agentAddress}: ${oldKarma} -> ${newKarma}`);
  }

  // Broadcast new rating submission
  public broadcastRatingSubmitted(raterAddress: string, ratedAddress: string, score: number, interactionHash: string) {
    const event: WebSocketEvent = {
      type: 'rating_submitted',
      data: {
        raterAddress,
        ratedAddress,
        score,
        interactionHash
      },
      timestamp: new Date().toISOString()
    };

    // Send to public room
    this.io.to('public').emit('rating_submitted', event);
    
    // Send to agent-specific subscribers
    this.io.to(`agent:${ratedAddress}`).emit('rating_submitted', event);
    this.io.to(`agent:${raterAddress}`).emit('rating_submitted', event);
    
    // Send to rating event subscribers
    this.io.to('events:rating_submitted').emit('rating_submitted', event);

    logger.info(`Broadcasted rating submission: ${raterAddress} rated ${ratedAddress} with score ${score}`);
  }

  // Broadcast new agent registration
  public broadcastAgentRegistered(agentAddress: string, metadata: any) {
    const event: WebSocketEvent = {
      type: 'agent_registered',
      data: {
        agentAddress,
        metadata
      },
      timestamp: new Date().toISOString()
    };

    // Send to public room
    this.io.to('public').emit('agent_registered', event);
    
    // Send to registration event subscribers
    this.io.to('events:agent_registered').emit('agent_registered', event);

    logger.info(`Broadcasted agent registration: ${agentAddress}`);
  }

  // Broadcast new proposal creation
  public broadcastProposalCreated(proposalId: string, title: string, proposer: string) {
    const event: WebSocketEvent = {
      type: 'proposal_created',
      data: {
        proposalId,
        title,
        proposer
      },
      timestamp: new Date().toISOString()
    };

    // Send to public room
    this.io.to('public').emit('proposal_created', event);
    
    // Send to proposal event subscribers
    this.io.to('events:proposal_created').emit('proposal_created', event);

    logger.info(`Broadcasted proposal creation: ${proposalId} by ${proposer}`);
  }

  // Broadcast vote cast
  public broadcastVoteCast(proposalId: string, voter: string, support: boolean, votingPower: number) {
    const event: WebSocketEvent = {
      type: 'vote_cast',
      data: {
        proposalId,
        voter,
        support,
        votingPower
      },
      timestamp: new Date().toISOString()
    };

    // Send to public room
    this.io.to('public').emit('vote_cast', event);
    
    // Send to proposal-specific subscribers
    this.io.to(`proposal:${proposalId}`).emit('vote_cast', event);
    
    // Send to vote event subscribers
    this.io.to('events:vote_cast').emit('vote_cast', event);

    logger.info(`Broadcasted vote cast: ${voter} voted ${support ? 'for' : 'against'} proposal ${proposalId}`);
  }

  // Send private message to specific user
  public sendToUser(userAddress: string, eventType: string, data: any) {
    const event: WebSocketEvent = {
      type: eventType as any,
      data,
      timestamp: new Date().toISOString()
    };

    this.io.to(`user:${userAddress}`).emit(eventType, event);
    logger.info(`Sent private message to ${userAddress}: ${eventType}`);
  }

  // Get connection statistics
  public getStats() {
    const totalConnections = this.io.sockets.sockets.size;
    const authenticatedConnections = this.connectedClients.size;
    const anonymousConnections = totalConnections - authenticatedConnections;

    return {
      totalConnections,
      authenticatedConnections,
      anonymousConnections,
      connectedUsers: Array.from(this.connectedClients.values())
    };
  }

  private isValidEventType(eventType: string): boolean {
    const validTypes = ['karma_updated', 'rating_submitted', 'agent_registered', 'proposal_created', 'vote_cast'];
    return validTypes.includes(eventType);
  }

  private isValidAddress(address: string): boolean {
    return typeof address === 'string' && address.startsWith('sei1') && address.length >= 39;
  }
}

export default WebSocketService;

