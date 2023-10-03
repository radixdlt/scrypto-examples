import { get } from 'svelte/store';
import { poolInfo, rdt } from '../store';

// ******** Fetch pool info from gateway api, save it in poolInfo store ********
export const fetchPoolInfo = async () => {
	const poolAddress = get(poolInfo).poolAddress;
	if (!poolAddress) return;

	// Fetch pool component info from gateway api
	const poolComponentInfo = await get(rdt)?.gatewayApi.state.getEntityDetailsVaultAggregated(
		poolAddress
	);

	console.log('PoolComponentInfo:', poolComponentInfo);

	// Set pool balance variable with gateway api getEntityDetailsVaultAggregated payload
	const resource0Balance = poolComponentInfo?.fungible_resources.items.find(
		(item) => item.resource_address === get(poolInfo).poolResources[0].address
	)?.vaults.items[0].amount;
	const resource1Balance = poolComponentInfo?.fungible_resources.items.find(
		(item) => item.resource_address === get(poolInfo).poolResources[1].address
	)?.vaults.items[0].amount;
	const poolUnitAddressData = poolComponentInfo?.metadata.items.find(
		(item) => item.key === 'pool_unit'
	)?.value.typed;
	const poolUnitAddress =
		poolUnitAddressData?.type === 'GlobalAddress' ? poolUnitAddressData.value : undefined;

	if (!poolUnitAddress) return;
	// Fetch pool unit info from gateway api
	const poolUnitInfo = await get(rdt)?.gatewayApi.state.getEntityDetailsVaultAggregated(
		poolUnitAddress
	);

	console.log('poolUnitInfo:', poolUnitInfo);

	// Save pool balance and pool unit info to store
	const address = poolUnitAddress;
	const balance = `${resource0Balance || '0'} ${get(poolInfo).poolResources[0].symbol} + ${
		resource1Balance || '0'
	} ${get(poolInfo).poolResources[1].symbol}`;
	const totalSupply =
		poolUnitInfo?.details?.type === 'FungibleResource' ? poolUnitInfo?.details?.total_supply : '';

	poolInfo.update((info) => ({
		...info,
		balance,
		poolUnit: {
			address,
			totalSupply
		}
	}));

	console.log('$PoolInfo:', get(poolInfo));
};
