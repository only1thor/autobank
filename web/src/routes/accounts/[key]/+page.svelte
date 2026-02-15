<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { Account, Transaction } from '$lib/api/types';
	import TransactionTable from '$lib/components/transactions/TransactionTable.svelte';
	import { ArrowLeft, RefreshCw, AlertCircle, CreditCard, Wallet } from 'lucide-svelte';

	let account = $state<Account | null>(null);
	let transactions = $state<Transaction[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const key = $derived(page.params.key);

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		if (!key) return;
		loading = true;
		error = null;
		try {
			const [accountData, txData] = await Promise.all([
				api.getAccount(key),
				api.getTransactions(key)
			]);
			account = accountData;
			transactions = txData.transactions;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
		} finally {
			loading = false;
		}
	}

	function formatCurrency(amount: number, currency: string): string {
		return new Intl.NumberFormat('nb-NO', {
			style: 'currency',
			currency
		}).format(amount);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/accounts" class="btn btn-ghost p-2">
			<ArrowLeft class="h-5 w-5" />
		</a>
		{#if account}
			<div>
				<h1 class="text-2xl font-bold text-gray-100">{account.name}</h1>
				<p class="text-gray-400">{account.accountNumber}</p>
			</div>
		{:else}
			<div>
				<h1 class="text-2xl font-bold text-gray-100">Account Details</h1>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<RefreshCw class="h-8 w-8 animate-spin text-primary-400" />
		</div>
	{:else if error}
		<div class="card p-6 bg-red-900/30 border-red-700">
			<div class="flex items-center gap-3 text-red-400">
				<AlertCircle class="h-5 w-5" />
				<p>{error}</p>
			</div>
			<button class="btn btn-secondary mt-4" onclick={loadData}>Retry</button>
		</div>
	{:else if account}
		<!-- Account summary -->
		<div class="card p-6">
			<div class="flex items-start gap-4">
				<div
					class="flex h-12 w-12 items-center justify-center rounded-full {account.type === 'CREDITCARD'
						? 'bg-purple-100 text-purple-600'
						: 'bg-primary-900/50 text-primary-400'}"
				>
					{#if account.type === 'CREDITCARD'}
						<CreditCard class="h-6 w-6" />
					{:else}
						<Wallet class="h-6 w-6" />
					{/if}
				</div>
				<div class="flex-1">
					<div class="flex items-center justify-between">
						<div>
							<h2 class="text-lg font-semibold text-gray-100">{account.name}</h2>
							<p class="text-sm text-gray-400">IBAN: {account.iban}</p>
						</div>
						<div class="text-right">
							<div class="text-2xl font-bold text-gray-100">
								{formatCurrency(account.balance, account.currencyCode)}
							</div>
							{#if account.availableBalance !== account.balance}
								<div class="text-sm text-gray-400">
									Available: {formatCurrency(account.availableBalance, account.currencyCode)}
								</div>
							{/if}
						</div>
					</div>
				</div>
			</div>

			<div class="mt-6 grid grid-cols-2 sm:grid-cols-4 gap-4 pt-4 border-t">
				<div>
					<div class="text-xs text-gray-400 uppercase">Product</div>
					<div class="text-sm font-medium">{account.productType}</div>
				</div>
				<div>
					<div class="text-xs text-gray-400 uppercase">Currency</div>
					<div class="text-sm font-medium">{account.currencyCode}</div>
				</div>
				<div>
					<div class="text-xs text-gray-400 uppercase">Transfer From</div>
					<div class="text-sm font-medium">
						{account.accountProperties.isTransferFromEnabled ? 'Yes' : 'No'}
					</div>
				</div>
				<div>
					<div class="text-xs text-gray-400 uppercase">Transfer To</div>
					<div class="text-sm font-medium">
						{account.accountProperties.isTransferToEnabled ? 'Yes' : 'No'}
					</div>
				</div>
			</div>
		</div>

		<!-- Transactions -->
		<div>
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-lg font-semibold text-gray-100">Transactions</h2>
				<button class="btn btn-secondary" onclick={loadData}>
					<RefreshCw class="h-4 w-4 mr-2" />
					Refresh
				</button>
			</div>
			<TransactionTable {transactions} />
		</div>
	{/if}
</div>
