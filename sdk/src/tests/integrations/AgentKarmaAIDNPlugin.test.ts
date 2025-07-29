import { AgentKarmaSDK } from "../../core/AgentKarmaSDK";
import { AgentKarmaAIDNPlugin } from "../../integrations/aidn";

// Mock de l'interface AIDN pour les tests
const mockAIDNClient = {
  on: jest.fn(),
};

const mockAgentInteractionData = {
  participants: ["aidn_agent_1", "aidn_agent_2"],
  type: "collaboration",
  metadata: { task: "AIDN collaborative task" },
};

describe("AgentKarmaAIDNPlugin Integration Tests", () => {
  let sdk: AgentKarmaSDK;
  let aidnPlugin: AgentKarmaAIDNPlugin;

  beforeEach(() => {
    // Mock du SDK AgentKarma pour éviter les appels réels à la blockchain pendant les tests unitaires
    sdk = {
      logInteraction: jest.fn().mockResolvedValue("txHash123"),
      getKarmaScore: jest.fn().mockResolvedValue({ score: 200, lastUpdated: Date.now() }),
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

    mockAIDNClient.on.mockClear();
    aidnPlugin = new AgentKarmaAIDNPlugin(sdk, mockAIDNClient as any);
  });

  it("should initialize AIDN event listeners", () => {
    expect(mockAIDNClient.on).toHaveBeenCalledWith("agent_interaction", expect.any(Function));
  });

  it("should log an interaction when AIDN agent_interaction event is triggered", async () => {
    const interactionHandler = mockAIDNClient.on.mock.calls.find(call => call[0] === "agent_interaction")[1];
    await interactionHandler(mockAgentInteractionData);
    expect(sdk.logInteraction).toHaveBeenCalledWith(mockAgentInteractionData);
  });

  it("should get agent karma score via AIDN plugin", async () => {
    const agentId = "sei1someagent";
    const karmaScore = await aidnPlugin.getAgentKarma(agentId);
    expect(sdk.getKarmaScore).toHaveBeenCalledWith(agentId);
    expect(karmaScore).toBe(200);
  });

  it("should handle error when logging interaction via AIDN", async () => {
    (sdk.logInteraction as jest.Mock).mockRejectedValueOnce(new Error("Failed to log"));
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    const interactionHandler = mockAIDNClient.on.mock.calls.find(call => call[0] === "agent_interaction")[1];
    await interactionHandler(mockAgentInteractionData);
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "AIDN: Failed to log interaction to Agent-Karma:",
      expect.any(Error)
    );
    consoleErrorSpy.mockRestore();
  });

  it("should handle error when getting karma score via AIDN", async () => {
    (sdk.getKarmaScore as jest.Mock).mockRejectedValueOnce(new Error("Failed to get karma"));
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    const karmaScore = await aidnPlugin.getAgentKarma("sei1someagent");
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "AIDN: Failed to get karma score for sei1someagent:",
      expect.any(Error)
    );
    expect(karmaScore).toBeUndefined();
    consoleErrorSpy.mockRestore();
  });
});

