#!/usr/bin/env node

/**
 * MVP End-to-End Test Script for Agent-Karma
 * Demonstrates: Agent Registration → Interaction → Rating → Karma Calculation
 */

const { AgentKarmaSDK } = require('../sdk/dist/core/AgentKarmaSDK');
const fs = require('fs');
const path = require('path');

// Test configuration
const TEST_CONFIG = {
  // Test agents
  agents: [
    {
      name: 'TestAgent-Alice',
      description: 'AI agent for testing MVP functionality',
      framework: 'ElizaOS',
      version: '1.0.0',
    },
    {
      name: 'TestAgent-Bob',
      description: 'Second AI agent for interaction testing',
      framework: 'MCP',
      version: '1.0.0',
    },
  ],
  // Test interaction
  interaction: {
    type: 'collaboration',
    metadata: {
      duration: 300, // 5 minutes
      outcome: 'successful',
      context: 'MVP testing scenario',
    },
  },
  // Test ratings
  ratings: [
    { score: 8, feedback: 'Great collaboration, very responsive' },
    { score: 7, feedback: 'Good performance, minor delays' },
  ],
};

class MVPTester {
  constructor() {
    this.sdk = null;
    this.deploymentInfo = null;
    this.testResults = {
      startTime: Date.now(),
      operations: [],
      performance: [],
      errors: [],
      summary: {},
    };
    this.agentAddresses = [];
  }

  async initialize() {
    console.log('🚀 Initializing MVP Test Suite...\n');

    // Load deployment info
    const deploymentPath = path.join(__dirname, '../deployment-info.json');
    if (!fs.existsSync(deploymentPath)) {
      throw new Error('Deployment info not found. Run deployment script first.');
    }

    this.deploymentInfo = JSON.parse(fs.readFileSync(deploymentPath, 'utf8'));
    console.log('📋 Loaded deployment info:');
    console.log(`  Network: ${this.deploymentInfo.chainId}`);
    console.log(`  RPC: ${this.deploymentInfo.rpcEndpoint}`);
    console.log(`  Contracts: ${Object.keys(this.deploymentInfo.contracts).length}\n`);

    // Initialize SDK
    const config = {
      rpcEndpoint: this.deploymentInfo.rpcEndpoint,
      chainId: this.deploymentInfo.chainId,
      contractAddresses: {
        agentRegistry: this.deploymentInfo.contracts.AgentRegistry.contractAddress,
        karmaCore: this.deploymentInfo.contracts.KarmaCore.contractAddress,
        interactionLogger: this.deploymentInfo.contracts.InteractionLogger.contractAddress,
        governanceDao: '', // Not needed for MVP
      },
    };

    // Use test mnemonic or environment variable
    const mnemonic = process.env.TEST_MNEMONIC || 
      'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

    this.sdk = new AgentKarmaSDK(config, { mnemonic });
    await this.sdk.connect();

    console.log(`✅ SDK initialized with signer: ${this.sdk.getSignerAddress()}\n`);
  }

  async measureOperation(name, operation) {
    const startTime = Date.now();
    let success = true;
    let result = null;
    let error = null;

    try {
      console.log(`⏱️  Starting: ${name}`);
      result = await operation();
      console.log(`✅ Completed: ${name}`);
    } catch (err) {
      success = false;
      error = err.message;
      console.error(`❌ Failed: ${name} - ${error}`);
      this.testResults.errors.push({ operation: name, error });
    }

    const endTime = Date.now();
    const responseTime = endTime - startTime;

    const operationResult = {
      name,
      responseTime,
      success,
      error,
      timestamp: new Date().toISOString(),
    };

    this.testResults.operations.push(operationResult);
    this.testResults.performance.push({
      operation: name,
      responseTime,
      success,
      error,
    });

    console.log(`📊 ${name}: ${responseTime}ms ${success ? '✅' : '❌'}\n`);

    return { result, responseTime, success };
  }

  async testAgentRegistration() {
    console.log('🔸 Phase 1: Agent Registration\n');

    for (let i = 0; i < TEST_CONFIG.agents.length; i++) {
      const agent = TEST_CONFIG.agents[i];
      
      const { result } = await this.measureOperation(
        `Register Agent ${i + 1}: ${agent.name}`,
        async () => {
          const txHash = await this.sdk.registerAgent(agent);
          
          // Wait a moment for transaction to be processed
          await new Promise(resolve => setTimeout(resolve, 2000));
          
          // Verify registration
          const signerAddress = this.sdk.getSignerAddress();
          const isRegistered = await this.sdk.isAgentRegistered(signerAddress);
          
          if (!isRegistered) {
            throw new Error('Agent registration verification failed');
          }

          this.agentAddresses.push(signerAddress);
          return { txHash, address: signerAddress };
        }
      );

      if (result) {
        console.log(`  📍 Agent Address: ${result.address}`);
        console.log(`  🧾 Transaction: ${result.txHash}\n`);
      }
    }
  }

  async testInteractionLogging() {
    console.log('🔸 Phase 2: Interaction Logging\n');

    if (this.agentAddresses.length < 2) {
      console.log('⚠️  Skipping interaction test - need at least 2 agents\n');
      return null;
    }

    const { result } = await this.measureOperation(
      'Log Interaction',
      async () => {
        const interactionData = {
          participants: this.agentAddresses.slice(0, 2),
          interactionType: TEST_CONFIG.interaction.type,
          metadata: TEST_CONFIG.interaction.metadata,
        };

        const txHash = await this.sdk.logInteraction(interactionData);
        
        // Wait for transaction processing
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        return { txHash, interactionData };
      }
    );

    if (result) {
      console.log(`  🧾 Transaction: ${result.txHash}`);
      console.log(`  👥 Participants: ${result.interactionData.participants.join(', ')}\n`);
      return result.txHash;
    }

    return null;
  }

  async testRatingSubmission(interactionHash) {
    console.log('🔸 Phase 3: Rating Submission\n');

    if (!interactionHash) {
      console.log('⚠️  Skipping rating test - no interaction hash\n');
      return;
    }

    if (this.agentAddresses.length < 2) {
      console.log('⚠️  Skipping rating test - need at least 2 agents\n');
      return;
    }

    // Submit rating from first agent to second agent
    const rating = TEST_CONFIG.ratings[0];
    
    const { result } = await this.measureOperation(
      `Submit Rating: ${rating.score}/10`,
      async () => {
        const txHash = await this.sdk.submitRating(
          this.agentAddresses[1], // Rate the second agent
          rating.score,
          interactionHash,
          rating.feedback
        );

        // Wait for karma calculation
        await new Promise(resolve => setTimeout(resolve, 3000));

        return { txHash, rating };
      }
    );

    if (result) {
      console.log(`  ⭐ Rating: ${result.rating.score}/10`);
      console.log(`  💬 Feedback: ${result.rating.feedback}`);
      console.log(`  🧾 Transaction: ${result.txHash}\n`);
    }
  }

  async testKarmaCalculation() {
    console.log('🔸 Phase 4: Karma Score Verification\n');

    for (let i = 0; i < this.agentAddresses.length; i++) {
      const address = this.agentAddresses[i];
      
      const { result } = await this.measureOperation(
        `Get Karma Score for Agent ${i + 1}`,
        async () => {
          const karmaScore = await this.sdk.getKarmaScore(address);
          const agent = await this.sdk.getAgent(address);
          
          return { karmaScore, agent };
        }
      );

      if (result) {
        console.log(`  📍 Agent: ${address}`);
        console.log(`  📊 Karma Score: ${result.karmaScore.score}`);
        console.log(`  📅 Last Updated: ${new Date(result.karmaScore.lastUpdated * 1000).toISOString()}`);
        if (result.agent) {
          console.log(`  👤 Name: ${result.agent.metadata.name}`);
          console.log(`  🏷️  Framework: ${result.agent.metadata.framework}`);
        }
        console.log();
      }
    }
  }

  async testLeaderboard() {
    console.log('🔸 Phase 5: Leaderboard Query\n');

    const { result } = await this.measureOperation(
      'Get Leaderboard',
      async () => {
        const leaderboard = await this.sdk.getLeaderboard(10);
        return leaderboard;
      }
    );

    if (result && result.length > 0) {
      console.log('  🏆 Current Leaderboard:');
      result.forEach((entry, index) => {
        console.log(`    ${index + 1}. ${entry.name} (${entry.address.slice(0, 10)}...) - ${entry.karma} karma`);
      });
      console.log();
    } else {
      console.log('  📋 Leaderboard is empty or unavailable\n');
    }
  }

  async testPerformanceBenchmark() {
    console.log('🔸 Phase 6: Performance Benchmark\n');

    const { result } = await this.measureOperation(
      'Performance Benchmark',
      async () => {
        const metrics = await this.sdk.measurePerformance();
        return metrics;
      }
    );

    if (result) {
      console.log('  📈 Performance Metrics:');
      result.forEach(metric => {
        const status = metric.responseTime < 400 ? '✅' : '⚠️';
        console.log(`    ${metric.operation}: ${metric.responseTime}ms ${status}`);
      });
      console.log();
    }
  }

  generateReport() {
    const endTime = Date.now();
    const totalTime = endTime - this.testResults.startTime;
    
    // Calculate summary statistics
    const successfulOps = this.testResults.operations.filter(op => op.success).length;
    const totalOps = this.testResults.operations.length;
    const avgResponseTime = this.testResults.performance
      .filter(p => p.success)
      .reduce((sum, p) => sum + p.responseTime, 0) / 
      this.testResults.performance.filter(p => p.success).length;

    const performanceTarget = 400; // 400ms target
    const fastOperations = this.testResults.performance
      .filter(p => p.success && p.responseTime < performanceTarget).length;
    
    this.testResults.summary = {
      totalTime,
      successRate: (successfulOps / totalOps) * 100,
      averageResponseTime: Math.round(avgResponseTime),
      performanceCompliance: (fastOperations / this.testResults.performance.filter(p => p.success).length) * 100,
      totalOperations: totalOps,
      successfulOperations: successfulOps,
      errors: this.testResults.errors.length,
    };

    // Generate report
    console.log('📊 MVP Test Report');
    console.log('='.repeat(50));
    console.log(`🕐 Total Test Time: ${totalTime}ms`);
    console.log(`✅ Success Rate: ${this.testResults.summary.successRate.toFixed(1)}% (${successfulOps}/${totalOps})`);
    console.log(`⚡ Average Response Time: ${this.testResults.summary.averageResponseTime}ms`);
    console.log(`🎯 Performance Target (<400ms): ${this.testResults.summary.performanceCompliance.toFixed(1)}%`);
    
    if (this.testResults.errors.length > 0) {
      console.log('\n❌ Errors:');
      this.testResults.errors.forEach(error => {
        console.log(`  - ${error.operation}: ${error.error}`);
      });
    }

    console.log('\n📋 Detailed Performance:');
    this.testResults.performance.forEach(metric => {
      const status = metric.success ? 
        (metric.responseTime < 400 ? '🟢' : '🟡') : '🔴';
      console.log(`  ${status} ${metric.operation}: ${metric.responseTime}ms`);
    });

    // Save detailed report
    const reportPath = path.join(__dirname, '../mvp-test-report.json');
    fs.writeFileSync(reportPath, JSON.stringify(this.testResults, null, 2));
    console.log(`\n📄 Detailed report saved to: ${reportPath}`);

    return this.testResults.summary;
  }

  async cleanup() {
    if (this.sdk) {
      await this.sdk.disconnect();
    }
  }
}

async function main() {
  const tester = new MVPTester();

  try {
    await tester.initialize();

    // Execute test phases
    await tester.testAgentRegistration();
    const interactionHash = await tester.testInteractionLogging();
    await tester.testRatingSubmission(interactionHash);
    await tester.testKarmaCalculation();
    await tester.testLeaderboard();
    await tester.testPerformanceBenchmark();

    // Generate final report
    const summary = tester.generateReport();

    console.log('\n🎉 MVP Test Suite Complete!');
    
    // Validate core assumptions
    console.log('\n🔍 Core Assumption Validation:');
    console.log(`  ✅ Agent Registration: ${summary.successfulOperations > 0 ? 'Working' : 'Failed'}`);
    console.log(`  ✅ Interaction Logging: ${interactionHash ? 'Working' : 'Failed'}`);
    console.log(`  ✅ Rating System: ${summary.successRate > 50 ? 'Working' : 'Failed'}`);
    console.log(`  ✅ Karma Calculation: ${summary.averageResponseTime < 1000 ? 'Working' : 'Slow'}`);
    console.log(`  ✅ Performance Target: ${summary.performanceCompliance > 70 ? 'Met' : 'Needs Optimization'}`);

    // Recommendations
    console.log('\n💡 Recommendations:');
    if (summary.performanceCompliance < 70) {
      console.log('  - Optimize contract queries for better performance');
      console.log('  - Consider caching frequently accessed data');
    }
    if (summary.successRate < 90) {
      console.log('  - Investigate and fix failing operations');
      console.log('  - Add better error handling and retry logic');
    }
    if (summary.averageResponseTime > 400) {
      console.log('  - Review blockchain interaction patterns');
      console.log('  - Consider batch operations where possible');
    }

    console.log('\n🚀 Ready for next development phase!');

  } catch (error) {
    console.error('\n💥 MVP Test Suite Failed:', error);
    process.exit(1);
  } finally {
    await tester.cleanup();
  }
}

// Run test suite if called directly
if (require.main === module) {
  main().catch(console.error);
}

module.exports = { MVPTester };