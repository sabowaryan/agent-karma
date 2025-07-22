// Sei Chain Configurations
export const PACIFIC_1_SEI_COSMOS_KIT_CHAIN = {
	chain_id: 'pacific-1',
	chain_name: 'sei',
	chain_type: 'cosmos' as const,
	pretty_name: 'Sei',
	bech32_prefix: 'sei',
	network_type: 'mainnet' as const,
	status: 'live' as const
};

export const ATLANTIC_2_SEI_COSMOS_KIT_CHAIN = {
	chain_id: 'atlantic-2',
	chain_name: 'seitestnet',
	chain_type: 'cosmos' as const,
	pretty_name: 'Sei Testnet',
	bech32_prefix: 'sei',
	network_type: 'testnet' as const,
	status: 'live' as const
};

export const ARCTIC_1_SEI_COSMOS_KIT_CHAIN = {
	chain_id: 'arctic-1',
	chain_name: 'seidevnet',
	chain_type: 'cosmos' as const,
	pretty_name: 'Sei Devnet',
	bech32_prefix: 'sei',
	network_type: 'devnet' as const,
	status: 'live' as const
};

export type Urls = {
	rpc: string;
	rest: string;
};

export const defaultUrls: { [chainName: string]: Urls } = {
	[PACIFIC_1_SEI_COSMOS_KIT_CHAIN.chain_id]: { rpc: 'https://rpc.sei-apis.com', rest: 'https://rest.sei-apis.com' },
	[ATLANTIC_2_SEI_COSMOS_KIT_CHAIN.chain_id]: { rpc: 'https://rpc-testnet.sei-apis.com', rest: 'https://rest-testnet.sei-apis.com' },
	[ARCTIC_1_SEI_COSMOS_KIT_CHAIN.chain_id]: { rpc: 'https://rpc-arctic-1.sei-apis.com', rest: 'https://rest-arctic-1.sei-apis.com' }
};

// Set your selected chain here
// To point to mainnet, use PACIFIC_1_SEI_COSMOS_KIT_CHAIN
export const selectedChain = ARCTIC_1_SEI_COSMOS_KIT_CHAIN;
