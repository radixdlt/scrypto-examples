<script lang="ts">
	import Button from '../../components/Button.svelte';
	import Card from '../../components/Card.svelte';
	import Input from '../../components/Input.svelte';
	import Page from '../../components/Page.svelte';
	import { poolInfo, rdt } from '../../store';
	import { getCreateResourcesManifest, getInstantiateManifest } from '../../manifests';
	import { onMount } from 'svelte';
	import { fetchPoolInfo } from '../../utils/fetchPoolInfo';

	onMount(() => {
		// Get and update pool info from gateway api when ever this page is mounted
		fetchPoolInfo();
	});

	// ******** Create resources usable in pool *********
	const handelCreatePoolResources = async () => {
		// Send manifest to wallet for signing
		const result = await $rdt?.walletApi.sendTransaction({
			transactionManifest: getCreateResourcesManifest()
		});
		if (!result) throw new Error('No response from Radix Wallet');
		if (result.isErr()) throw result.error;
		console.log('Create Resources WalletSDK Result: ', result.value);

		const transactionStatus = await $rdt?.gatewayApi.transaction.getStatus(
			result.value.transactionIntentHash
		);
		console.log('Create Resources TransactionApi transaction/status:', transactionStatus?.status);

		// Fetch the transaction status from the Gateway API
		const committedDetails = await $rdt?.gatewayApi.transaction.getCommittedDetails(
			result.value.transactionIntentHash
		);
		console.log('Create Resources getCommittedDetails:', committedDetails);

		// Set pool resource addresses in store
		const resource1Address = committedDetails?.transaction.affected_global_entities?.[2];
		const resource2Address = committedDetails?.transaction.affected_global_entities?.[3];
		if (!resource1Address || !resource2Address) throw new Error('No resource address returned');
		poolInfo.update((info) => ({
			...info,
			poolResources: [{ address: resource1Address }, { address: resource2Address }]
		}));
	};

	// ******** Instantiate component and fetch component and pool unit addresses *********
	const handleInstantiate = async () => {
		// Send manifest to extension for signing
		const result = await $rdt?.walletApi.sendTransaction({
			transactionManifest: getInstantiateManifest()
		});
		if (!result) throw new Error('No response from Radix Wallet');
		if (result.isErr()) throw result.error;
		console.log('Instantiate WalletSDK Result: ', result.value);

		// Fetch the transaction status from the Gateway API
		const transactionStatus = await $rdt?.gatewayApi.transaction.getStatus(
			result.value.transactionIntentHash
		);
		console.log('Instantiate TransactionApi transaction/status:', transactionStatus?.status);

		// Fetch component address from gateway api
		const committedDetails = await $rdt?.gatewayApi.transaction.getCommittedDetails(
			result.value.transactionIntentHash
		);
		console.log('Instantiate getCommittedDetails:', committedDetails);

		// Set Radiswap componentAddress variable, with gateway api getCommittedDetails payload, in store
		const componentAddress = committedDetails?.transaction.affected_global_entities?.[1];
		poolInfo.update((info) => ({ ...info, componentAddress }));

		// Set poolAddress variable, with gateway api getCommittedDetails payload, in store.
		// This is for the pool component instantiated with and used by the Radiswap component
		const poolAddress = committedDetails?.transaction.affected_global_entities?.[2];
		poolInfo.update((info) => ({ ...info, poolAddress }));

		// Set poolUnitAddress variable, with gateway api getCommittedDetails payload, in store.
		// This is not strictly necessary as it's also set a when fetching pool info in the layout component
		const poolUnitAddress = committedDetails?.transaction.affected_global_entities?.[3];
		poolInfo.update((info) => ({ ...info, poolUnit: { address: poolUnitAddress || '' } }));
	};
</script>

<Page title="Admin">
	<Card>
		<h2>Pool Resources</h2>
		<Button
			disabled={!!($poolInfo.poolResources[0].address && $poolInfo.poolResources[1].address)}
			on:click={handelCreatePoolResources}>Create Resources</Button
		>
		<Input
			name="resource1address"
			label="Pool Resource 1 Address"
			value={$poolInfo.poolResources[0].address}
			readonly
		/>
		<Input
			name="resource1address"
			label="Pool Resource 2 Address"
			value={$poolInfo.poolResources[1].address}
			readonly
		/>
		<h2>Component and Pool</h2>
		<Button on:click={handleInstantiate}>Instantiate</Button>

		<Input
			name="pool"
			label="Pool"
			value={$poolInfo.poolResources.map((info) => `${info.name} (${info.symbol})`).join(' and ')}
			readonly
		/>
		<Input
			name="radiswapAddress"
			label="Radiswap Component Address"
			value={$poolInfo.componentAddress}
			readonly
		/>
		<Input name="poolAddress" label="Pool Address" value={$poolInfo.poolAddress} readonly />
		<Input name="poolBalance" label="Pool Balance" value={$poolInfo.balance} readonly />
		<Input
			name="poolUnitAddress"
			label="Pool Unit Address"
			value={$poolInfo.poolUnit?.address}
			readonly
		/>
		<Input
			name="poolUnitSupply"
			label="Pool Unit Total Supply"
			value={$poolInfo.poolUnit?.totalSupply
				? $poolInfo.poolUnit?.totalSupply + ' Pool Units'
				: $poolInfo.poolUnit?.totalSupply}
			readonly
		/>
	</Card>
</Page>

<style lang="scss">
	h2 {
		margin-bottom: 1rem;
	}
</style>
