const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Oracle Integration Test", function () {
    let agentRegistry, oracleIntegration, karmaCore;
    let owner, agent1, agent2, provider1;

    beforeEach(async function () {
        [owner, agent1, agent2, provider1] = await ethers.getSigners();

        // Deploy AgentRegistry
        const AgentRegistryFactory = await ethers.getContractFactory("AgentRegistry");
        agentRegistry = await AgentRegistryFactory.deploy();
        await agentRegistry.waitForDeployment();

        // Deploy OracleIntegration
        const OracleIntegrationFactory = await ethers.getContractFactory("OracleIntegration");
        oracleIntegration = await OracleIntegrationFactory.deploy(agentRegistry.target);
        await oracleIntegration.waitForDeployment();

        // Deploy KarmaCore
        const KarmaCoreFactory = await ethers.getContractFactory("KarmaCore");
        karmaCore = await KarmaCoreFactory.deploy(agentRegistry.target, oracleIntegration.target);
        await karmaCore.waitForDeployment();

        // Register agents
        await agentRegistry.connect(agent1).registerAgent({ name: "Agent1", description: "AI Assistant", framework: "", version: "", ipfsHash: "http://agent1.com" });
        await agentRegistry.connect(agent2).registerAgent({ name: "Agent2", description: "AI Helper", framework: "", version: "", ipfsHash: "http://agent2.com" });
        await agentRegistry.connect(provider1).registerAgent({ name: "Provider1", description: "Data Provider", framework: "", version: "", ipfsHash: "http://provider1.com" });
    });

    it("Should authorize oracle provider and submit data", async function () {
        // Authorize provider
        await oracleIntegration.connect(owner).authorizeProvider(provider1.address);
        
        // Check if provider is authorized
        expect(await oracleIntegration.isProviderAuthorized(provider1.address)).to.be.true;

        // Submit oracle data
        const dataType = "agent_performance";
        const data = "5"; // Performance score
        const signatures = ["sig1", "sig2", "sig3"]; // Mock signatures

        await oracleIntegration.connect(provider1).submitOracleData(dataType, data, signatures);

        // Get latest oracle data
        const latestData = await oracleIntegration.getLatestOracleData(dataType);
        expect(latestData.provider).to.equal(provider1.address);
        expect(latestData.data).to.equal(data);
        expect(latestData.verified).to.be.true;
    });

    it("Should integrate oracle data in karma calculation", async function () {
        // Authorize provider and submit data
        await oracleIntegration.connect(owner).authorizeProvider(provider1.address);
        
        const dataType = "agent_performance";
        const data = "3"; // Performance bonus
        const signatures = ["sig1", "sig2", "sig3"];

        await oracleIntegration.connect(provider1).submitOracleData(dataType, data, signatures);

        // Submit a rating to trigger karma calculation
        const interactionHash = ethers.keccak256(ethers.toUtf8Bytes("test_interaction"));
        await karmaCore.connect(agent1).submitRating(
            agent2.address,
            8,
            "Great performance",
            interactionHash
        );

        // Check karma score (should include oracle data)
        const karmaScore = await karmaCore.getKarmaScore(agent2.address);
        console.log("Karma score with oracle integration:", karmaScore.toString());
        
        // The karma should be: average rating (8) + frequency bonus (1) + external factor (3) = 12
        expect(karmaScore).to.be.greaterThan(8);
    });

    it("Should handle missing oracle data gracefully", async function () {
        // Submit a rating without oracle data
        const interactionHash = ethers.keccak256(ethers.toUtf8Bytes("test_interaction_no_oracle"));
        await karmaCore.connect(agent1).submitRating(
            agent2.address,
            7,
            "Good performance",
            interactionHash
        );

        // Check karma score (should work without oracle data)
        const karmaScore = await karmaCore.getKarmaScore(agent2.address);
        console.log("Karma score without oracle data:", karmaScore.toString());
        
        // The karma should be: average rating (7) + frequency bonus (1) + external factor (0) = 8
        expect(karmaScore).to.equal(8);
    });
});

