<script lang="ts">
	import Page from '../../components/Page.svelte';
	import Button from '../../components/Button.svelte';
	import Card from '../../components/Card.svelte';
	import Input from '../../components/Input.svelte';
	import { poolInfo, rdt } from '../../store';
	import { getAddLiquidityManifest, getRemoveLiquidityManifest } from '../../manifests';
	import { transactionPreview } from '../../utils/transactionPreview';

	let accountPoolUnitBalance: string = '';
	let depOrWit: 'Deposit' | 'Withdraw' = 'Deposit';
	let fromAmount1: string = '';
	let fromAmount2: string = '';
	let toAmount1: string = '';
	let toAmount2: string = '';
	let toAmount3: string = '';
	$: fromResource1 = $poolInfo.poolResources[0];
	$: fromResource2 = $poolInfo.poolResources[1];
	$: toResource1 = $poolInfo.poolUnit;
	$: toResource2 = $poolInfo.poolResources[0];
	$: toResource3 = $poolInfo.poolResources[1];

	// Get and update pool unit balance whenever the pool unit address changes
	$: if ($poolInfo.poolUnit?.address) {
		fetchAccountPoolUnitBalance();
	}

	// ******** Get and update pool unit balance ********
	const fetchAccountPoolUnitBalance = async () => {
		const result = await $rdt?.gatewayApi.state.getEntityDetailsVaultAggregated(
			$rdt.walletApi.getWalletData().accounts[0].address
		);

		accountPoolUnitBalance =
			result?.fungible_resources.items.find(
				(item) => item.resource_address === $poolInfo.poolUnit?.address
			)?.vaults.items[0].amount || '0';
	};

	// ******** Set all To and From resources to there correct values when changing between deposit and withdraw ********
	const handleChangeDepositWithdraw = () => {
		depOrWit = depOrWit === 'Deposit' ? 'Withdraw' : 'Deposit';
		toAmount1 = '';
		toAmount2 = '';
		if (depOrWit === 'Deposit' && $poolInfo.poolUnit) {
			fromResource1 = $poolInfo.poolResources[0];
			fromResource2 = $poolInfo.poolResources[1];
			toResource1 = $poolInfo.poolUnit;
		} else if ($poolInfo.poolUnit) {
			fromResource1 = $poolInfo.poolUnit;
		}
		forecastReceiveAmount();
	};

	// ******** Clear amounts ********
	const clearAmounts = () => {
		fromAmount1 = '';
		fromAmount2 = '';
		toAmount1 = '';
		toAmount2 = '';
		toAmount3 = '';
	};

	// ******** Send add liquidity transaction to wallet clear inputs and update pool unit balance on completion ********
	const handleDeposit = async () => {
		const result = await $rdt?.walletApi.sendTransaction({
			transactionManifest: getAddLiquidityManifest({ amount1: fromAmount1, amount2: fromAmount2 })
		});
		if (!result) throw new Error('No response from Radix Wallet');
		if (result.isErr()) throw result.error;

		console.log('Deposit Result: ', result.value);

		clearAmounts();
		fetchAccountPoolUnitBalance();
	};

	// ******** Send remove liquidity transaction to wallet then clear inputs and update pool unit balance on completion ********
	const handleWithdraw = async () => {
		const result = await $rdt?.walletApi.sendTransaction({
			transactionManifest: getRemoveLiquidityManifest({ amount: fromAmount1 })
		});
		if (!result) throw new Error('No response from Radix Wallet');
		if (result.isErr()) throw result.error;

		console.log('Withdraw Result: ', result.value);

		clearAmounts();
		fetchAccountPoolUnitBalance();
	};

	// ******** Forecast add or remove liquidity transaction result ********
	const forecastReceiveAmount = async () => {
		if (
			$rdt?.gatewayApi &&
			$poolInfo.poolUnit?.address &&
			fromAmount1 &&
			(depOrWit === 'Withdraw' || fromAmount2)
		) {
			// Create add or remove liquidity manifest
			const manifest =
				depOrWit === 'Deposit'
					? getAddLiquidityManifest({ amount1: fromAmount1, amount2: fromAmount2 })
					: getRemoveLiquidityManifest({ amount: fromAmount1 });

			// Get predicted transaction result of the manifest from gateway ap
			const result = await transactionPreview(manifest);

			console.log('pool forecast', result);

			// The returned result is not fully typed, so we have to use @ts-ignore
			// @ts-ignore
			if (result.receipt.status === 'Failed') {
				toAmount1 = 'This transaction will not succeed';
				toAmount2 = 'This transaction will not succeed';
				toAmount3 = '...';
			}
			// @ts-ignore
			if (result.receipt.status === 'Succeeded') {
				const finalResourceChanges =
					// @ts-ignore
					result.resource_changes[result.resource_changes.length - 1]?.resource_changes;
				// Set toAmounts from the transaction preview result
				toAmount1 =
					finalResourceChanges.find(
						(change: any) => change.resource_address === toResource1?.address
					)?.amount || '0';
				toAmount2 =
					finalResourceChanges.find(
						(change: any) => change.resource_address === toResource2?.address
					)?.amount || '0';
				toAmount3 =
					finalResourceChanges.find(
						(change: any) => change.resource_address === toResource3?.address
					)?.amount || '0';
			}
		}
	};
</script>

<Page title="Pool">
	<Card>
		<h2>Add or Remove Liquidity</h2>
		<div class="item">
			<Input name="poolBalance" value={accountPoolUnitBalance} readonly />
			<Input name="poolUnit" label="your pool units" value="Pool Units" readonly />
		</div>

		<Button on:click={handleChangeDepositWithdraw}>Deposit ⇄ Withdraw</Button>

		<div class="item">
			<Input
				name="from1"
				type="number"
				bind:value={fromAmount1}
				on:change={forecastReceiveAmount}
			/>
			<Input
				name="fromResource1"
				label="you send"
				value={fromResource1.symbol || 'Pool Units'}
				readonly
			/>
		</div>
		{#if depOrWit === 'Deposit'}
			<div class="item">
				<Input
					name="from2"
					type="number"
					bind:value={fromAmount2}
					on:change={forecastReceiveAmount}
				/>
				<Input name="fromResource2" value={fromResource2.symbol} readonly />
			</div>

			<div class="item">
				<Input name="to" value={toAmount1} readonly />
				<Input
					name="toResource"
					label="you receive"
					value={toResource1?.symbol || 'Pool Units'}
					readonly
				/>
			</div>
		{/if}

		<div class="item">
			<Input name="to2" value={toAmount2} readonly />
			<Input
				name="toResource2"
				label={depOrWit === 'Deposit' ? 'You Keep' : 'You Receive'}
				value={toResource2?.symbol}
				readonly
			/>
		</div>
		<div class="item">
			<Input name="to3" value={toAmount3} readonly />
			<Input name="toResource3" value={toResource3?.symbol} readonly />
		</div>

		<Button
			on:click={() => {
				if (depOrWit === 'Deposit') handleDeposit();
				else handleWithdraw();
			}}>{depOrWit}</Button
		>
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
