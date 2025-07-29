const hre = require("hardhat");

async function main() {
  // Deploy AgentRegistry
  const AgentRegistry = await hre.ethers.getContractFactory("AgentRegistry");
  const agentRegistry = await AgentRegistry.deploy();
  await agentRegistry.waitForDeployment();
  console.log(`AgentRegistry deployed to ${agentRegistry.target}`);

  // Deploy KarmaCore
  const KarmaCore = await hre.ethers.getContractFactory("KarmaCore");
  const karmaCore = await KarmaCore.deploy(agentRegistry.target);
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

  // Deploy OracleIntegration
  const OracleIntegration = await hre.ethers.getContractFactory("OracleIntegration");
  const oracleIntegration = await OracleIntegration.deploy(agentRegistry.target);
  await oracleIntegration.waitForDeployment();
  console.log(`OracleIntegration deployed to ${oracleIntegration.target}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});


