import { SDKError, RetryConfig } from "../types";

export class AgentKarmaError extends Error implements SDKError {
  code: string;
  details?: any;
  retryable: boolean;

  constructor(message: string, code: string, retryable: boolean = false, details?: any) {
    super(message);
    this.name = "AgentKarmaError";
    this.code = code;
    this.retryable = retryable;
    this.details = details;

    // Set the prototype explicitly.
    Object.setPrototypeOf(this, AgentKarmaError.prototype);
  }
}

export async function withRetry<T>(fn: () => Promise<T>, config: RetryConfig): Promise<T> {
  let retries = 0;
  let delay = config.baseDelay;

  while (retries < config.maxRetries) {
    try {
      return await fn();
    } catch (error: any) {
      if (error instanceof AgentKarmaError && !error.retryable) {
        throw error; // Do not retry non-retryable errors
      }

      console.warn(`Operation failed, retrying in ${delay / 1000}s... (Attempt ${retries + 1}/${config.maxRetries})`, error);
      await new Promise(resolve => setTimeout(resolve, delay));
      retries++;
      delay = Math.min(delay * config.backoffMultiplier, config.maxDelay);
    }
  }
  throw new AgentKarmaError("Operation failed after multiple retries", "MAX_RETRIES_EXCEEDED");
}


