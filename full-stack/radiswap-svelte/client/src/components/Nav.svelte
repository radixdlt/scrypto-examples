<nav>
	<a href="/">√<span class="overline">&nbsp;Radiswap</span></a>
	<div class="links">
		<a href="/swap">Swap</a>
		<a href="/pool">Pool</a>
		<a href="/admin">Admin</a>
	</div>
	<div class="connect-button-container">
		<radix-connect-button />
	</div>
</nav>

<style lang="scss">
	.overline {
		text-decoration: overline;
	}
	nav {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.5rem 1rem;
		margin: -0.5rem;
		height: 3rem;
		background-color: #000;
		color: #fff;
		a {
			color: #fff;
			text-decoration: none;
			&:hover {
				color: #fff;
				text-decoration: underline;
			}
		}
		.links {
			display: flex;
			gap: 3rem;
		}
		.connect-button-container {
			display: flex;
			justify-content: center;
			align-items: center;
			border: 1.5px solid #fff;
		}
	}
</style>
