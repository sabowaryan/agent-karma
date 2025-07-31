import { io, Socket } from 'socket.io-client';
import { WebSocketEvent } from '../types';

const WS_URL = import.meta.env.VITE_WS_URL || 'http://localhost:3000';

class WebSocketService {
  private socket: Socket | null = null;
  private eventListeners: Map<string, Set<(data: any) => void>> = new Map();
  private isConnected = false;

  connect(token?: string): void {
    if (this.socket) {
      this.disconnect();
    }

    const auth = token ? { token } : {};
    
    this.socket = io(WS_URL, {
      auth,
      transports: ['websocket', 'polling'],
      autoConnect: true,
    });

    this.setupEventHandlers();
  }

  disconnect(): void {
    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
      this.isConnected = false;
    }
  }

  private setupEventHandlers(): void {
    if (!this.socket) return;

    this.socket.on('connect', () => {
      console.log('WebSocket connected');
      this.isConnected = true;
    });

    this.socket.on('disconnect', (reason) => {
      console.log('WebSocket disconnected:', reason);
      this.isConnected = false;
    });

    this.socket.on('connected', (data) => {
      console.log('WebSocket welcome message:', data);
    });

    this.socket.on('error', (error) => {
      console.error('WebSocket error:', error);
    });

    // Handle specific events
    this.socket.on('karma_updated', (event: WebSocketEvent) => {
      this.notifyListeners('karma_updated', event.data);
    });

    this.socket.on('rating_submitted', (event: WebSocketEvent) => {
      this.notifyListeners('rating_submitted', event.data);
    });

    this.socket.on('agent_registered', (event: WebSocketEvent) => {
      this.notifyListeners('agent_registered', event.data);
    });

    this.socket.on('proposal_created', (event: WebSocketEvent) => {
      this.notifyListeners('proposal_created', event.data);
    });

    this.socket.on('vote_cast', (event: WebSocketEvent) => {
      this.notifyListeners('vote_cast', event.data);
    });
  }

  private notifyListeners(eventType: string, data: any): void {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(data);
        } catch (error) {
          console.error(`Error in WebSocket listener for ${eventType}:`, error);
        }
      });
    }
  }

  // Subscribe to specific events
  subscribe(eventTypes: string[]): void {
    if (this.socket && this.isConnected) {
      this.socket.emit('subscribe', eventTypes);
    }
  }

  // Unsubscribe from events
  unsubscribe(eventTypes: string[]): void {
    if (this.socket && this.isConnected) {
      this.socket.emit('unsubscribe', eventTypes);
    }
  }

  // Subscribe to agent-specific updates
  subscribeToAgent(agentAddress: string): void {
    if (this.socket && this.isConnected) {
      this.socket.emit('subscribe_agent', agentAddress);
    }
  }

  // Subscribe to proposal-specific updates
  subscribeToProposal(proposalId: string): void {
    if (this.socket && this.isConnected) {
      this.socket.emit('subscribe_proposal', proposalId);
    }
  }

  // Add event listener
  addEventListener(eventType: string, listener: (data: any) => void): void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    this.eventListeners.get(eventType)!.add(listener);
  }

  // Remove event listener
  removeEventListener(eventType: string, listener: (data: any) => void): void {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.delete(listener);
      if (listeners.size === 0) {
        this.eventListeners.delete(eventType);
      }
    }
  }

  // Send ping to check connection
  ping(): void {
    if (this.socket && this.isConnected) {
      this.socket.emit('ping');
    }
  }

  // Get connection status
  getConnectionStatus(): boolean {
    return this.isConnected;
  }

  // Get socket ID
  getSocketId(): string | undefined {
    return this.socket?.id;
  }
}

export const webSocketService = new WebSocketService();
export default webSocketService;

