<script lang="ts">
	import { DataRequestBuilder, RadixDappToolkit, createLogger } from '@radixdlt/radix-dapp-toolkit';
	import { onMount } from 'svelte';
	import { poolInfo, rdt } from '../store';
	import config from '../config.json';
	import Nav from '../components/Nav.svelte';
	import { fetchPoolInfo } from '../utils/fetchPoolInfo';

	onMount(() => {
		// Initialize Radix Dapp Toolkit for connect button, wallet and gateway api usage
		$rdt = (
			RadixDappToolkit({
				dAppDefinitionAddress: config.dAppDefinitionAddress,
				networkId: 2,
				logger: createLogger(0)
			})
		);
		$rdt?.buttonApi.setMode('light');
		$rdt?.buttonApi.setTheme('white');
		$rdt?.walletApi.setRequestData(DataRequestBuilder.accounts().exactly(1));

		fetchResourceInfo($poolInfo.poolResources[0].address);
		fetchResourceInfo($poolInfo.poolResources[1].address);
		fetchPoolInfo();
	});

	// ******** Using given resource addresses, fetch resource info from gateway api then save it in poolInfo store ********
	const fetchResourceInfo = async (address: string) => {
		// Fetch resource metadata from gateway api
		const result = await $rdt?.gatewayApi.state.getEntityMetadata(address);

		console.log('Fetch ResourceInfo:', result);

		// Get resource name and symbol from metadata
		const nameData = result?.items.find((item) => item.key === 'name')?.value.typed;
		const name: string = nameData?.type === 'String' ? nameData.value : '';
		const symbolData = result?.items.find((item) => item.key === 'symbol')?.value.typed;
		const symbol: string = symbolData?.type === 'String' ? symbolData.value : '';
		const resourceInfo = { name, symbol, address };

		// Set pool resources in store
		poolInfo.update((info) => ({
			...info,
			poolResources: info.poolResources.map((resource) =>
				resource.address === resourceInfo.address ? resourceInfo : resource
			)
		}));
	};
</script>

<div>
	<Nav />
	<slot />
</div>

<style lang="scss">
	@import url('https://fonts.googleapis.com/css2?family=Roboto&display=swap');

	:global(*) {
		box-sizing: border-box;
	}
	div {
		font-family: Roboto, sans-serif;
	}
</style>
