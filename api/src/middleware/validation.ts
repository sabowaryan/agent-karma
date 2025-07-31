import { Request, Response, NextFunction } from 'express';
import Joi from 'joi';

// Validation schemas
export const schemas = {
  registerAgent: Joi.object({
    metadata: Joi.object({
      name: Joi.string().required().min(1).max(100),
      description: Joi.string().optional().max(500),
      capabilities: Joi.array().items(Joi.string()).optional(),
      ipfsHash: Joi.string().optional()
    }).required()
  }),

  submitRating: Joi.object({
    ratedAddress: Joi.string().required(),
    score: Joi.number().integer().min(1).max(10).required(),
    interactionHash: Joi.string().required(),
    context: Joi.string().optional().max(500)
  }),

  createProposal: Joi.object({
    title: Joi.string().required().min(5).max(200),
    description: Joi.string().required().min(10).max(2000)
  }),

  vote: Joi.object({
    proposalId: Joi.string().required(),
    support: Joi.boolean().required()
  }),

  pagination: Joi.object({
    page: Joi.number().integer().min(1).default(1),
    limit: Joi.number().integer().min(1).max(100).default(20)
  })
};

// Validation middleware factory
export const validate = (schema: Joi.ObjectSchema) => {
  return (req: Request, res: Response, next: NextFunction) => {
    const { error, value } = schema.validate(req.body);
    
    if (error) {
      return res.status(400).json({
        success: false,
        error: `Validation error: ${error.details[0].message}`,
        timestamp: new Date().toISOString()
      });
    }
    
    req.body = value;
    next();
  };
};

// Query validation middleware
export const validateQuery = (schema: Joi.ObjectSchema) => {
  return (req: Request, res: Response, next: NextFunction) => {
    const { error, value } = schema.validate(req.query);
    
    if (error) {
      return res.status(400).json({
        success: false,
        error: `Query validation error: ${error.details[0].message}`,
        timestamp: new Date().toISOString()
      });
    }
    
    req.query = value;
    next();
  };
};

// Address validation middleware
export const validateAddress = (req: Request, res: Response, next: NextFunction) => {
  const { address } = req.params;
  
  if (!address || !address.startsWith('sei1') || address.length < 39) {
    return res.status(400).json({
      success: false,
      error: 'Invalid Sei address format',
      timestamp: new Date().toISOString()
    });
  }
  
  next();
};

