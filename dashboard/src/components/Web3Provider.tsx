'use client';

import { ReactNode } from 'react';
import { Buffer } from 'buffer';

import { ChainProvider } from '@cosmos-kit/react';
import { wallets as keplrWallets } from '@cosmos-kit/keplr-extension';
import { wallets as leapWallets } from '@cosmos-kit/leap-extension';
import { selectedChain } from '../constants';

interface Web3ProviderProps {
	children: ReactNode;
}

function Web3Provider({ children }: Web3ProviderProps) {
	// Buffer polyfill
	if (typeof window !== 'undefined') {
		window.Buffer = window.Buffer ?? Buffer;
	}

	// Define your supported wallets here
	const wallets = [...keplrWallets, ...leapWallets];

	// Define the list of sei chain you would like to connect to here.
	const chains = [selectedChain];

	// Create a simple asset list for Sei
	const assetLists = [
		{
			chain_name: selectedChain.chain_name,
			assets: [
				{
					description: 'The native staking token of Sei.',
					denom_units: [
						{
							denom: 'usei',
							exponent: 0
						},
						{
							denom: 'sei',
							exponent: 6
						}
					],
					base: 'usei',
					name: 'Sei',
					display: 'sei',
					symbol: 'SEI',
					type_asset: 'sdk.coin' as const,
					logo_URIs: {
						png: 'https://raw.githubusercontent.com/cosmos/chain-registry/master/sei/images/sei.png',
						svg: 'https://raw.githubusercontent.com/cosmos/chain-registry/master/sei/images/sei.svg'
					}
				}
			]
		}
	];

	return (
		<ChainProvider 
			chains={chains} 
			assetLists={assetLists} 
			wallets={wallets}
			throwErrors={false}
		>
			{children}
		</ChainProvider>
	);
}

export default Web3Provider;
