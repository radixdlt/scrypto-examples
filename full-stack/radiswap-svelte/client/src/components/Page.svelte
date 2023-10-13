<script lang="ts">
	export let title: string = '';
</script>

<main>
	<h1>{title}</h1>
	<slot />
</main>

<style lang="scss">
	main {
		display: flex;
		gap: 2rem;
		flex-wrap: wrap;
		justify-content: center;
		padding: 1rem 2rem;
		h1 {
			margin-bottom: 2rem;
			width: 100%;
		}
	}
</style>
