<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let disabled = false;

	const dispatch = createEventDispatcher();
	const handleClick = () => {
		dispatch('click');
	};
</script>

<button {disabled} on:click={handleClick}>
	<slot />
</button>

<style lang="scss">
	button {
		font-weight: 600;
		width: 100%;
		padding: 0.5rem;
		margin: 0.5rem 0 1.5rem;
		border: 1px solid #fff;
		border-radius: 0.25rem;
		background-color: #000;
		color: #fff;
		cursor: pointer;

		&:disabled {
			border: 1px solid #000;
			border-radius: 0.25rem;
			background-color: #fff;
			color: #000;
			cursor: not-allowed;
		}
	}
</style>
