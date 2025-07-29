import { AgentKarmaSDK } from "../../core/AgentKarmaSDK";
import { AgentKarmaElizaOSPlugin } from "../../integrations/elizaos";

// Mock de l'interface ElizaOS pour les tests
const mockElizaOSContext = {
  agentId: "elizaos_agent_1",
  otherAgentId: "elizaos_agent_2",
  type: "message_exchange",
  metadata: { message: "Hello, ElizaOS!" },
};

describe("AgentKarmaElizaOSPlugin Integration Tests", () => {
  let sdk: AgentKarmaSDK;
  let elizaOSPlugin: AgentKarmaElizaOSPlugin;

  beforeEach(() => {
    // Mock du SDK AgentKarma pour éviter les appels réels à la blockchain pendant les tests unitaires
    sdk = {
      logInteraction: jest.fn().mockResolvedValue("txHash123"),
      getKarmaScore: jest.fn().mockResolvedValue({ score: 100, lastUpdated: Date.now() }),
      connect: jest.fn().mockResolvedValue(undefined),
      disconnect: jest.fn().mockResolvedValue(undefined),
      isConnected: jest.fn().mockReturnValue(true),
      canSign: jest.fn().mockReturnValue(true),
      getSignerAddress: jest.fn().mockReturnValue("sei1testsigner"),
      registerAgent: jest.fn().mockResolvedValue("txHash456"),
      submitRating: jest.fn().mockResolvedValue("txHash789"),
      getAgent: jest.fn().mockResolvedValue({ address: "sei1testsigner", metadata: { name: "TestAgent" } }),
      isAgentRegistered: jest.fn().mockResolvedValue(true),
      measurePerformance: jest.fn().mockResolvedValue([]),
    } as any; // Cast pour éviter les erreurs de type dues au mock partiel

    elizaOSPlugin = new AgentKarmaElizaOSPlugin(sdk);
  });

  it("should log an interaction via ElizaOS plugin", async () => {
    await elizaOSPlugin.onInteraction(mockElizaOSContext);
    expect(sdk.logInteraction).toHaveBeenCalledWith({
      participants: [mockElizaOSContext.agentId, mockElizaOSContext.otherAgentId],
      interactionType: mockElizaOSContext.type,
      metadata: mockElizaOSContext.metadata,
    });
  });

  it("should get agent karma score via ElizaOS plugin", async () => {
    const agentId = "sei1someagent";
    const karmaScore = await elizaOSPlugin.getAgentKarma(agentId);
    expect(sdk.getKarmaScore).toHaveBeenCalledWith(agentId);
    expect(karmaScore).toBe(100);
  });

  it("should handle error when logging interaction", async () => {
    (sdk.logInteraction as jest.Mock).mockRejectedValueOnce(new Error("Failed to log"));
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    await elizaOSPlugin.onInteraction(mockElizaOSContext);
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "ElizaOS: Failed to log interaction to Agent-Karma:",
      expect.any(Error)
    );
    consoleErrorSpy.mockRestore();
  });

  it("should handle error when getting karma score", async () => {
    (sdk.getKarmaScore as jest.Mock).mockRejectedValueOnce(new Error("Failed to get karma"));
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    const karmaScore = await elizaOSPlugin.getAgentKarma("sei1someagent");
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "ElizaOS: Failed to get karma score for sei1someagent:",
      expect.any(Error)
    );
    expect(karmaScore).toBeUndefined();
    consoleErrorSpy.mockRestore();
  });
});


