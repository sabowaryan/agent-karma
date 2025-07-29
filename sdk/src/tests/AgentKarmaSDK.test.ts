import { AgentKarmaSDK } from "../core/AgentKarmaSDK";
import { AgentKarmaConfig, AgentMetadata, InteractionData } from "../types";

// Mock des adresses de contrat pour les tests
const mockContractAddresses = {
  agentRegistry: "sei1agentregistrymockaddress",
  karmaCore: "sei1karmacoremockaddress",
  interactionLogger: "sei1interactionloggermockaddress",
  governanceDao: "sei1governancedaomockaddress",
};

const mockConfig: AgentKarmaConfig = {
  rpcEndpoint: "https://rpc.testnet.sei.io",
  chainId: "atlantic-2",
  contractAddresses: mockContractAddresses,
};

// Utiliser un mnémonique de test (à ne jamais utiliser en production !)
const testMnemonic = "notice grid cable desk parade obtain latin velvet sport adult december vibrant";

describe("AgentKarmaSDK Integration Tests", () => {
  let sdk: AgentKarmaSDK;

  beforeAll(async () => {
    sdk = new AgentKarmaSDK(mockConfig, { mnemonic: testMnemonic });
    await sdk.connect();
  });

  afterAll(async () => {
    await sdk.disconnect();
  });

  it("should connect successfully", () => {
    expect(sdk.isConnected()).toBe(true);
    expect(sdk.canSign()).toBe(true);
    expect(sdk.getSignerAddress()).toBeDefined();
  });

  it("should register an agent", async () => {
    const agentMetadata: AgentMetadata = {
      name: `TestAgent-${Date.now()}`,
      description: "A test agent for SDK integration tests.",
      framework: "Jest",
      version: "1.0.0",
    };

    const txHash = await sdk.registerAgent(agentMetadata);
    expect(txHash).toBeDefined();
    expect(typeof txHash).toBe("string");
    console.log("Register Agent Tx Hash:", txHash);
  });

  it("should log an interaction", async () => {
    const interactionData: InteractionData = {
      participants: [sdk.getSignerAddress()!, "sei1anotheragent"],
      interactionType: "test_interaction",
      metadata: { context: "SDK test interaction" },
    };

    const txHash = await sdk.logInteraction(interactionData);
    expect(txHash).toBeDefined();
    expect(typeof txHash).toBe("string");
    console.log("Log Interaction Tx Hash:", txHash);
  });

  it("should submit a rating", async () => {
    const ratedAgent = "sei1ratedagent";
    const score = 7;
    const interactionHash = "0x" + Math.random().toString(16).substr(2, 32); // Mock hash

    const txHash = await sdk.submitRating(ratedAgent, score, interactionHash);
    expect(txHash).toBeDefined();
    expect(typeof txHash).toBe("string");
    console.log("Submit Rating Tx Hash:", txHash);
  });

  it("should get agent information", async () => {
    // Note: This test assumes the agent from registerAgent test is available
    // In a real scenario, you\'d register a known agent or query an existing one.
    const agentAddress = sdk.getSignerAddress()!;
    const agent = await sdk.getAgent(agentAddress);
    expect(agent).toBeDefined();
    expect(agent?.address).toBe(agentAddress);
    expect(agent?.metadata.name).toContain("TestAgent");
  });

  it("should get karma score", async () => {
    const agentAddress = sdk.getSignerAddress()!;
    const karma = await sdk.getKarmaScore(agentAddress);
    expect(karma).toBeDefined();
    expect(typeof karma.score).toBe("number");
    expect(typeof karma.lastUpdated).toBe("number");
  });

  it("should check if agent is registered", async () => {
    const agentAddress = sdk.getSignerAddress()!;
    const isRegistered = await sdk.isAgentRegistered(agentAddress);
    expect(isRegistered).toBe(true);

    const nonExistentAgent = "sei1nonexistent";
    const isNonExistentRegistered = await sdk.isAgentRegistered(nonExistentAgent);
    expect(isNonExistentRegistered).toBe(false);
  });

  it("should measure performance", async () => {
    const measurements = await sdk.measurePerformance();
    expect(measurements.length).toBeGreaterThan(0);
    measurements.forEach(m => {
      expect(m.operation).toBeDefined();
      expect(m.responseTime).toBeGreaterThanOrEqual(0);
    });
    console.log("Performance Measurements:", measurements);
  });
});


