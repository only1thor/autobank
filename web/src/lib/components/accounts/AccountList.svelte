<script lang="ts">
	import type { Account } from '$lib/api/types';
	import AccountCard from './AccountCard.svelte';

	interface Props {
		accounts: Account[];
		onselect?: (account: Account) => void;
	}

	let { accounts, onselect }: Props = $props();
</script>

<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
	{#each accounts as account (account.key)}
		<AccountCard {account} onclick={() => onselect?.(account)} />
	{/each}
</div>
