const hre = require("hardhat");

async function main() {
  let oracleIntegrationInstance; // Renamed to avoid conflict

  // Deploy AgentRegistry
  const AgentRegistry = await hre.ethers.getContractFactory("AgentRegistry");
  const agentRegistry = await AgentRegistry.deploy();
  await agentRegistry.waitForDeployment();
  console.log(`AgentRegistry deployed to ${agentRegistry.target}`);

  // Deploy OracleIntegration
  const OracleIntegration = await hre.ethers.getContractFactory("OracleIntegration");
  oracleIntegrationInstance = await OracleIntegration.deploy(agentRegistry.target);
  await oracleIntegrationInstance.waitForDeployment();
  console.log(`OracleIntegration deployed to ${oracleIntegrationInstance.target}`);

  // Deploy KarmaCore
  const KarmaCore = await hre.ethers.getContractFactory("KarmaCore");
  const karmaCore = await KarmaCore.deploy(agentRegistry.target, oracleIntegrationInstance.target);
  await karmaCore.waitForDeployment();
  console.log(`KarmaCore deployed to ${karmaCore.target}`);

  // Deploy InteractionLogger
  const InteractionLogger = await hre.ethers.getContractFactory("InteractionLogger");
  const interactionLogger = await InteractionLogger.deploy(agentRegistry.target);
  await interactionLogger.waitForDeployment();
  console.log(`InteractionLogger deployed to ${interactionLogger.target}`);

  // Deploy GovernanceDAO
  const GovernanceDAO = await hre.ethers.getContractFactory("GovernanceDAO");
  const governanceDAO = await GovernanceDAO.deploy(agentRegistry.target, karmaCore.target);
  await governanceDAO.waitForDeployment();
  console.log(`GovernanceDAO deployed to ${governanceDAO.target}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


