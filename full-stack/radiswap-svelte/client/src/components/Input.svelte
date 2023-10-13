<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { HTMLInputTypeAttribute } from 'svelte/elements';

	export let type: HTMLInputTypeAttribute = 'text';
	export let name: string;
	export let label: string = '';
	export let value: any;
	export let readonly: boolean = false;

	// `value` can't be bound for inputs as a change in type changes how it's handled. This manual function is a workaround.
	const onInput = (e: Event & { currentTarget: EventTarget & HTMLInputElement }) => {
		value = type.match(/^(number|range)$/) ? +e.currentTarget?.value : e.currentTarget?.value;
	};

	const dispatch = createEventDispatcher();
	const onChange = (e: Event) => {
		dispatch('change', e);
	};
</script>

<div>
	{#if label}
		<label for={name}>{label}</label>
	{/if}
	<input {type} id={name} {name} {value} {readonly} on:change={onChange} on:input={onInput} />
</div>

<style lang="scss">
	div {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		width: 100%;
		label {
			align-self: flex-end;
			text-transform: uppercase;
			font-size: 0.75rem;
			font-weight: bold;
			margin-bottom: 0.5rem;
		}
		input {
			width: 100%;
			margin-bottom: 1rem;
			padding: 0.5rem;
			border: 1px solid #000;
			border-radius: 0.25rem;
		}
	}
</style>
