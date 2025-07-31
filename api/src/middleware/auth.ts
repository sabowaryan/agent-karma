import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { AuthToken } from '../types';

const JWT_SECRET = process.env.JWT_SECRET || 'agent-karma-secret-key';

// Extend Request interface to include user
declare global {
  namespace Express {
    interface Request {
      user?: AuthToken;
    }
  }
}

// Generate JWT token for an agent address
export const generateToken = (address: string): string => {
  return jwt.sign(
    { address },
    JWT_SECRET,
    { expiresIn: '24h' }
  );
};

// Verify JWT token middleware
export const authenticateToken = (req: Request, res: Response, next: NextFunction) => {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1]; // Bearer TOKEN

  if (!token) {
    return res.status(401).json({
      success: false,
      error: 'Access token required',
      timestamp: new Date().toISOString()
    });
  }

  jwt.verify(token, JWT_SECRET, (err, decoded) => {
    if (err) {
      return res.status(403).json({
        success: false,
        error: 'Invalid or expired token',
        timestamp: new Date().toISOString()
      });
    }

    req.user = decoded as AuthToken;
    next();
  });
};

// Optional authentication middleware (doesn't fail if no token)
export const optionalAuth = (req: Request, res: Response, next: NextFunction) => {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];

  if (token) {
    jwt.verify(token, JWT_SECRET, (err, decoded) => {
      if (!err) {
        req.user = decoded as AuthToken;
      }
    });
  }

  next();
};

// Middleware to check if user has minimum karma
export const requireMinimumKarma = (minKarma: number) => {
  return async (req: Request, res: Response, next: NextFunction) => {
    if (!req.user) {
      return res.status(401).json({
        success: false,
        error: 'Authentication required',
        timestamp: new Date().toISOString()
      });
    }

    try {
      // TODO: Implement karma check with SDK
      // const karma = await agentKarmaSDK.getKarmaScore(req.user.address);
      // if (karma < minKarma) {
      //   return res.status(403).json({
      //     success: false,
      //     error: `Minimum karma of ${minKarma} required`,
      //     timestamp: new Date().toISOString()
      //   });
      // }

      next();
    } catch (error) {
      return res.status(500).json({
        success: false,
        error: 'Failed to verify karma requirements',
        timestamp: new Date().toISOString()
      });
    }
  };
};

