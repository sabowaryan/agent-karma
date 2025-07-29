import { AgentKarmaSDK } from "../../core/AgentKarmaSDK";
import { AgentKarmaMCPPlugin } from "../../integrations/mcp";

// Mock de l'interface MCP pour les tests
const mockMCPServer = {
  on: jest.fn(),
};

const mockInteractionData = {
  participants: ["mcp_agent_1", "mcp_agent_2"],
  type: "data_exchange",
  metadata: { data: "MCP data exchange" },
};

describe("AgentKarmaMCPPlugin Integration Tests", () => {
  let sdk: AgentKarmaSDK;
  let mcpPlugin: AgentKarmaMCPPlugin;

  beforeEach(() => {
    // Mock du SDK AgentKarma pour éviter les appels réels à la blockchain pendant les tests unitaires
    sdk = {
      logInteraction: jest.fn().mockResolvedValue("txHash123"),
      getKarmaScore: jest.fn().mockResolvedValue({ score: 150, lastUpdated: Date.now() }),
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

    mockMCPServer.on.mockClear();
    mcpPlugin = new AgentKarmaMCPPlugin(sdk, mockMCPServer as any);
  });

  it("should initialize MCP event listeners", () => {
    expect(mockMCPServer.on).toHaveBeenCalledWith("interaction", expect.any(Function));
  });

  it("should log an interaction when MCP interaction event is triggered", async () => {
    const interactionHandler = mockMCPServer.on.mock.calls.find(call => call[0] === "interaction")[1];
    await interactionHandler(mockInteractionData);
    expect(sdk.logInteraction).toHaveBeenCalledWith(mockInteractionData);
  });

  it("should get agent karma score via MCP plugin", async () => {
    const agentId = "sei1someagent";
    const karmaScore = await mcpPlugin.getAgentKarma(agentId);
    expect(sdk.getKarmaScore).toHaveBeenCalledWith(agentId);
    expect(karmaScore).toBe(150);
  });

  it("should handle error when logging interaction via MCP", async () => {
    (sdk.logInteraction as jest.Mock).mockRejectedValueOnce(new Error("Failed to log"));
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    const interactionHandler = mockMCPServer.on.mock.calls.find(call => call[0] === "interaction")[1];
    await interactionHandler(mockInteractionData);
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "MCP: Failed to log interaction to Agent-Karma:",
      expect.any(Error)
    );
    consoleErrorSpy.mockRestore();
  });

  it("should handle error when getting karma score via MCP", async () => {
    (sdk.getKarmaScore as jest.Mock).mockRejectedValueOnce(new Error("Failed to get karma"));
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    const karmaScore = await mcpPlugin.getAgentKarma("sei1someagent");
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "MCP: Failed to get karma score for sei1someagent:",
      expect.any(Error)
    );
    expect(karmaScore).toBeUndefined();
    consoleErrorSpy.mockRestore();
  });
});

