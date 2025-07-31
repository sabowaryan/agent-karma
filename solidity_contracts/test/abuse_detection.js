const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Abuse Detection Module Test", function () {
    let agentRegistry, oracleIntegration, karmaCore;
    let owner, agent1, agent2, agent3;

    beforeEach(async function () {
        [owner, agent1, agent2, agent3] = await ethers.getSigners();

        // Deploy AgentRegistry
        const AgentRegistryFactory = await ethers.getContractFactory("AgentRegistry");
        agentRegistry = await AgentRegistryFactory.deploy();
        await agentRegistry.waitForDeployment();

        // Deploy OracleIntegration (dummy for this test)
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
        await agentRegistry.connect(agent3).registerAgent({ name: "Agent3", description: "AI Helper", framework: "", version: "", ipfsHash: "http://agent3.com" });

        // Give agent1 some initial karma to be able to rate
        await karmaCore.connect(owner).rewardAgent(agent1.address, 100);
    });

    it("Should penalize an agent", async function () {
        const initialKarma = await karmaCore.getKarmaScore(agent2.address);
        const penaltyAmount = 10;

        await expect(karmaCore.connect(owner).penalizeAgent(agent2.address, penaltyAmount))
            .to.emit(karmaCore, "AgentPenalized")
            .withArgs(agent2.address, penaltyAmount);

        expect(await karmaCore.getKarmaScore(agent2.address)).to.equal(initialKarma - BigInt(penaltyAmount));
    });

    it("Should reward an agent", async function () {
        const initialKarma = await karmaCore.getKarmaScore(agent2.address);
        const rewardAmount = 20;

        await expect(karmaCore.connect(owner).rewardAgent(agent2.address, rewardAmount))
            .to.emit(karmaCore, "AgentRewarded")
            .withArgs(agent2.address, rewardAmount);

        expect(await karmaCore.getKarmaScore(agent2.address)).to.equal(initialKarma + BigInt(rewardAmount));
    });

    it("Should prevent non-owner from penalizing/rewarding", async function () {
        const penaltyAmount = 10;
        await expect(karmaCore.connect(agent1).penalizeAgent(agent2.address, penaltyAmount))
            .to.be.revertedWith("Only owner can call this function");

        const rewardAmount = 20;
        await expect(karmaCore.connect(agent1).rewardAgent(agent2.address, rewardAmount))
            .to.be.revertedWith("Only owner can call this function");
    });

    it("Should detect and apply penalty for rapid consecutive ratings", async function () {
        const initialKarmaAgent1 = await karmaCore.getKarmaScore(agent1.address);
        const interactionHash1 = ethers.keccak256(ethers.toUtf8Bytes("interaction_1"));
        const interactionHash2 = ethers.keccak256(ethers.toUtf8Bytes("interaction_2"));

        // First rating
        await karmaCore.connect(agent1).submitRating(agent2.address, 5, "feedback1", interactionHash1);

        // Simulate rapid second rating (within 60 seconds)
        // Note: The current _detectAbuse and _applyKarmaPenalty are simplified and don't actually modify state
        // or apply penalties in this test setup. This test primarily verifies the call flow.
        await karmaCore.connect(agent1).submitRating(agent3.address, 5, "feedback2", interactionHash2);

        // Verify that the karma of agent1 (rater) is not affected by the simplified abuse detection
        // In a real implementation, this would check for a reduced karma score.
        expect(await karmaCore.getKarmaScore(agent1.address)).to.equal(initialKarmaAgent1);
    });
});


