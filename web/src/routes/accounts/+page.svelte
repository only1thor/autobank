<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { Account } from '$lib/api/types';
	import AccountList from '$lib/components/accounts/AccountList.svelte';
	import { RefreshCw, AlertCircle } from 'lucide-svelte';
	import { goto } from '$app/navigation';

	let accounts = $state<Account[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		await loadAccounts();
	});

	async function loadAccounts() {
		loading = true;
		error = null;
		try {
			const data = await api.getAccounts();
			accounts = data.accounts;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load accounts';
		} finally {
			loading = false;
		}
	}

	function handleSelect(account: Account) {
		goto(`/accounts/${account.key}`);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-100">Accounts</h1>
			<p class="text-gray-400">Your bank accounts and credit cards</p>
		</div>
		<button class="btn btn-secondary" onclick={loadAccounts} disabled={loading}>
			<RefreshCw class="h-4 w-4 mr-2 {loading ? 'animate-spin' : ''}" />
			Refresh
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<RefreshCw class="h-8 w-8 animate-spin text-primary-500" />
		</div>
	{:else if error}
		<div class="card p-6 bg-red-900/30 border-red-700">
			<div class="flex items-center gap-3 text-red-400">
				<AlertCircle class="h-5 w-5" />
				<p>{error}</p>
			</div>
			<button class="btn btn-secondary mt-4" onclick={loadAccounts}>Retry</button>
		</div>
	{:else}
		<AccountList {accounts} onselect={handleSelect} />
	{/if}
</div>
