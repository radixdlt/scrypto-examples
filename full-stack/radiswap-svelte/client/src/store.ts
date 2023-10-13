import type { RadixDappToolkit } from '@radixdlt/radix-dapp-toolkit';
import { writable } from 'svelte/store';
import config from './config.json';

export type Resource = {
	address: string;
	name?: string;
	symbol?: string;
	totalSupply?: string;
};

export type PoolInfo = {
	balance?: string;
	componentAddress?: string;
	poolAddress?: string;
	poolUnit?: Resource;
	poolResources: Resource[];
};

// ********* Radix Dapp Toolkit store *********
// As the Radix Dapp Toolkit produces a value that you only wait for once this rdt store could alternatively be a promise.
export const rdt = writable<null | RadixDappToolkit>(null);

// ********* Radiswap component, pool, and resource info store *********
export const poolInfo = writable<PoolInfo>({
	componentAddress: config.componentAddress,
	poolAddress: config.poolAddress,
	poolResources: [
		{ address: config.poolResourceAddress1 },
		{ address: config.poolResourceAddress2 }
	]
});
