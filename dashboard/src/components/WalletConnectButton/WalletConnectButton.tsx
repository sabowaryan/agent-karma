import { useChain } from '@cosmos-kit/react';

import '@interchain-ui/react/styles';

import { selectedChain } from '../../constants';

// Helper function to truncate Sei addresses
const truncateSeiAddress = (address: string, startLength = 6, endLength = 4): string => {
	if (!address) return '';
	if (address.length <= startLength + endLength) return address;
	return `${address.slice(0, startLength)}...${address.slice(-endLength)}`;
};

export function WalletConnectButton() {
	const { isWalletConnected, address, connect, openView } = useChain(selectedChain.chain_name);

	const truncatedSeiAddr = address ? truncateSeiAddress(address) : '';

	return (
		<button className="primary" onClick={isWalletConnected ? openView : connect}>
			{isWalletConnected ? truncatedSeiAddr : 'Connect'}
		</button>
	);
}
