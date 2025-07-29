const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Agent Karma MVP", function () {
    let agentRegistry;
    let karmaCore;
    let interactionLogger;
    let owner;
    let agent1;
    let agent2;

    beforeEach(async function () {
        [owner, agent1, agent2] = await ethers.getSigners();

        // Deploy AgentRegistry
        const AgentRegistryFactory = await ethers.getContractFactory("AgentRegistry");
        agentRegistry = await AgentRegistryFactory.deploy();
        await agentRegistry.waitForDeployment();

        // Deploy KarmaCore
        const KarmaCoreFactory = await ethers.getContractFactory("KarmaCore");
        karmaCore = await KarmaCoreFactory.deploy(agentRegistry.target);
        await karmaCore.waitForDeployment();

        // Deploy InteractionLogger
        const InteractionLoggerFactory = await ethers.getContractFactory("InteractionLogger");
        interactionLogger = await InteractionLoggerFactory.deploy(agentRegistry.target);
        await interactionLogger.waitForDeployment();
    });

    describe("MVP Flow", function () {
        it("should allow agent registration, interaction logging, rating submission, and karma calculation", async function () {
            // 1. Register Agent1
            const agent1Metadata = {
                name: "AgentOne",
                description: "A test agent",
                framework: "ElizaOS",
                version: "1.0",
                ipfsHash: ""
            };
            await agentRegistry.connect(agent1).registerAgent(agent1Metadata);
            expect(await agentRegistry.isAgentRegistered(agent1.address)).to.be.true;
            console.log(`Agent1 (${agent1.address}) registered.`);

            // 2. Register Agent2
            const agent2Metadata = {
                name: "AgentTwo",
                description: "Another test agent",
                framework: "MCP",
                version: "1.0",
                ipfsHash: ""
            };
            await agentRegistry.connect(agent2).registerAgent(agent2Metadata);
            expect(await agentRegistry.isAgentRegistered(agent2.address)).to.be.true;
            console.log(`Agent2 (${agent2.address}) registered.`);

            // 3. Log an interaction between Agent1 and Agent2
            const participants = [agent1.address, agent2.address];
            const interactionType = "conversation";
            const interactionMetadata = {
                duration: 60,
                outcome: "successful",
                context: ""
            };
            const tx = await interactionLogger.connect(agent1).logInteraction(
                participants,
                interactionType,
                interactionMetadata
            );
            const receipt = await tx.wait();
            const interactionHash = receipt.logs[0].args[0]; // Get the hash from the event
            expect(await interactionLogger.verifyInteraction(interactionHash)).to.be.true;
            console.log(`Interaction logged with hash: ${interactionHash}`);

            // 4. Agent1 submits a rating for Agent2
            const score = 9;
            const feedback = "Great collaboration!";
            await karmaCore.connect(agent1).submitRating(agent2.address, score, feedback, interactionHash);
            console.log(`Agent1 rated Agent2 with score ${score}.`);

            // 5. Check Agent2's karma score
            const agent2Karma = await karmaCore.getKarmaScore(agent2.address);
            expect(agent2Karma).to.be.above(0);
            console.log(`Agent2's karma score: ${agent2Karma}`);

            // 6. Get Agent2's karma calculation details
            const karmaCalculation = await karmaCore.calculateKarma(agent2.address);
            console.log(`Agent2's karma calculation details:`);
            console.log(`  Current Score: ${karmaCalculation.currentScore}`);
            console.log(`  Average Rating: ${karmaCalculation.factors.averageRating}`);
            console.log(`  Rating Count: ${karmaCalculation.factors.ratingCount}`);
        });
    });
});


