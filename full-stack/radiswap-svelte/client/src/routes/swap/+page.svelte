<script lang="ts">
	import Button from '../../components/Button.svelte';
	import Card from '../../components/Card.svelte';
	import Input from '../../components/Input.svelte';
	import Page from '../../components/Page.svelte';
	import { getSwapManifest } from '../../manifests';
	import { poolInfo, rdt, type Resource } from '../../store';
	import { transactionPreview } from '../../utils/transactionPreview';

	let fromResource: Resource = { name: 'loading...', address: '', symbol: 'loading...' };
	let fromAmount: string = '0';
	let toResource: Resource = { name: 'loading...', address: '', symbol: 'loading...' };
	let toAmount: string = '';

	// Get pool resources from store
	$: if ($poolInfo.poolResources.length === 2) {
		fromResource = $poolInfo.poolResources[0];
		toResource = $poolInfo.poolResources[1];
	}

	// ******** Swap To and From resources ********
	const handleChangeDirection = () => {
		const _fromToken = fromResource;
		fromResource = toResource;
		toResource = _fromToken;
		forecastSwap();
	};

	// ******** Send swap transaction and clear inputs on completion ********
	const sendSwapTransaction = async () => {
		// Send swap manifest to wallet for signing
		const result = await $rdt?.walletApi.sendTransaction({
			transactionManifest: getSwapManifest({
				resourceAddress: fromResource.address,
				amount: fromAmount
			})
		});
		if (!result) throw new Error('No response from Radix Wallet');
		if (result.isErr()) throw result.error;

		console.log('Swap Result: ', result.value);

		// Clear inputs
		fromAmount = '';
		toAmount = '';
	};

	// ******** Forecast swap transaction result ********
	const forecastSwap = async () => {
		if ($rdt?.gatewayApi && $poolInfo.poolUnit?.address && fromAmount && fromResource.address) {
			// Create swap manifest
			const manifest = getSwapManifest({
				amount: fromAmount,
				resourceAddress: fromResource.address
			});

			// Get predicted transaction result of the manifest from gateway api
			const result = await transactionPreview(manifest);
			console.log('swap forecast', result);

			// The returned result is not fully typed, so we have to use @ts-ignore
			// @ts-ignore
			if (result.receipt.status === 'Failed') {
				toAmount = 'This transaction will not succeed';
			}
			// @ts-ignore
			if (result.receipt.status === 'Succeeded') {
				const finalResourceChanges =
					// @ts-ignore
					result.resource_changes[result.resource_changes.length - 1]?.resource_changes;
				// Set toAmount from the transaction preview result
				toAmount =
					finalResourceChanges.find((change: any) => change.resource_address === toResource.address)
						?.amount || '0';
			}
		}
	};
</script>

<Page title="Swap">
	<Card>
		<h2>Swap {fromResource.name} to {toResource.name}</h2>
		<Button on:click={handleChangeDirection}>Change direction ⇅</Button>
		<div>
			<div class="item">
				<Input type="number" name="from" bind:value={fromAmount} on:change={forecastSwap} />
				<Input name="fromToken" label="you pay" value={fromResource.symbol} readonly />
			</div>
			<div class="item">
				<Input name="to" value={toAmount} readonly />
				<Input name="toToken" label="you receive" value={toResource.symbol} readonly />
			</div>
			<Button on:click={sendSwapTransaction}>Submit</Button>
		</div>
	</Card>
</Page>

<style lang="scss">
	.item {
		display: grid;
		grid-template-columns: 2fr 1fr;
		gap: 1rem;
		align-items: end;
		max-width: 32rem;
	}
</style>
