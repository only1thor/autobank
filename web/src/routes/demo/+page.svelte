<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { DemoAccount, DemoStatus } from '$lib/api/types';
	import { RefreshCw, AlertCircle, Plus, CheckCircle, Beaker } from 'lucide-svelte';

	let demoStatus = $state<DemoStatus | null>(null);
	let accounts = $state<DemoAccount[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let submitting = $state(false);
	let successMessage = $state<string | null>(null);

	// Form state
	let selectedAccountKey = $state('');
	let description = $state('');
	let amount = $state('');
	let isSettled = $state(true);

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		try {
			const [statusRes, accountsRes] = await Promise.all([
				api.getDemoStatus(),
				api.getDemoAccounts()
			]);
			demoStatus = statusRes;
			accounts = accountsRes.accounts;
			if (accounts.length > 0 && !selectedAccountKey) {
				selectedAccountKey = accounts[0].key;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load demo data';
		} finally {
			loading = false;
		}
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();
		if (!selectedAccountKey || !description || !amount) return;

		submitting = true;
		error = null;
		successMessage = null;

		try {
			const result = await api.createDemoTransaction({
				accountKey: selectedAccountKey,
				description,
				amount: parseFloat(amount),
				isSettled
			});

			if (result.success) {
				successMessage = result.message;
				// Reset form
				description = '';
				amount = '';
				isSettled = true;
				// Reload accounts to show updated balance
				await loadData();
			} else {
				error = result.message;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create transaction';
		} finally {
			submitting = false;
		}
	}

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('nb-NO', {
			style: 'currency',
			currency: 'NOK'
		}).format(value);
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-2">
				<Beaker class="h-6 w-6 text-amber-600" />
				<h1 class="text-2xl font-bold text-gray-900">Demo Mode</h1>
			</div>
			<p class="text-gray-500">Create test transactions to test your automation rules</p>
		</div>
		<button class="btn btn-secondary" onclick={loadData} disabled={loading}>
			<RefreshCw class="h-4 w-4 mr-2 {loading ? 'animate-spin' : ''}" />
			Refresh
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<RefreshCw class="h-8 w-8 animate-spin text-primary-600" />
		</div>
	{:else if !demoStatus?.enabled}
		<div class="card p-6 bg-amber-50 border-amber-200">
			<div class="flex items-center gap-3 text-amber-700">
				<AlertCircle class="h-5 w-5" />
				<div>
					<p class="font-medium">Demo mode is not enabled</p>
					<p class="text-sm mt-1">Start the server with the <code class="bg-amber-100 px-1 rounded">--demo</code> flag to enable demo mode.</p>
				</div>
			</div>
		</div>
	{:else}
		{#if error}
			<div class="card p-4 bg-red-50 border-red-200">
				<div class="flex items-center gap-3 text-red-700">
					<AlertCircle class="h-5 w-5" />
					<p>{error}</p>
				</div>
			</div>
		{/if}

		{#if successMessage}
			<div class="card p-4 bg-green-50 border-green-200">
				<div class="flex items-center gap-3 text-green-700">
					<CheckCircle class="h-5 w-5" />
					<p>{successMessage}</p>
				</div>
			</div>
		{/if}

		<div class="grid gap-6 lg:grid-cols-2">
			<!-- Create Transaction Form -->
			<div class="card p-6">
				<h2 class="text-lg font-semibold text-gray-900 mb-4">Create Transaction</h2>
				<form onsubmit={handleSubmit} class="space-y-4">
					<div>
						<label for="account" class="block text-sm font-medium text-gray-700 mb-1">
							Account
						</label>
						<select
							id="account"
							bind:value={selectedAccountKey}
							class="input w-full"
							required
						>
							{#each accounts as account}
								<option value={account.key}>
									{account.name} ({account.accountNumber})
								</option>
							{/each}
						</select>
					</div>

					<div>
						<label for="description" class="block text-sm font-medium text-gray-700 mb-1">
							Description
						</label>
						<input
							id="description"
							type="text"
							bind:value={description}
							class="input w-full"
							placeholder="e.g., Netflix subscription"
							required
						/>
					</div>

					<div>
						<label for="amount" class="block text-sm font-medium text-gray-700 mb-1">
							Amount (NOK)
						</label>
						<input
							id="amount"
							type="number"
							step="0.01"
							bind:value={amount}
							class="input w-full"
							placeholder="e.g., -149.00 (negative for expenses)"
							required
						/>
						<p class="text-xs text-gray-500 mt-1">Use negative values for expenses</p>
					</div>

					<div class="flex items-center gap-2">
						<input
							id="isSettled"
							type="checkbox"
							bind:checked={isSettled}
							class="h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
						/>
						<label for="isSettled" class="text-sm text-gray-700">
							Transaction is settled (not pending)
						</label>
					</div>

					<button
						type="submit"
						class="btn btn-primary w-full"
						disabled={submitting || !selectedAccountKey || !description || !amount}
					>
						{#if submitting}
							<RefreshCw class="h-4 w-4 mr-2 animate-spin" />
							Creating...
						{:else}
							<Plus class="h-4 w-4 mr-2" />
							Create Transaction
						{/if}
					</button>
				</form>
			</div>

			<!-- Demo Accounts -->
			<div class="card p-6">
				<h2 class="text-lg font-semibold text-gray-900 mb-4">Demo Accounts</h2>
				<div class="space-y-3">
					{#each accounts as account}
						<div class="p-3 rounded-lg border border-gray-200 {selectedAccountKey === account.key ? 'ring-2 ring-primary-500 border-primary-500' : ''}">
							<div class="flex justify-between items-start">
								<div>
									<p class="font-medium text-gray-900">{account.name}</p>
									<p class="text-sm text-gray-500">{account.accountNumber}</p>
									<span class="inline-block mt-1 text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600">
										{account.accountType}
									</span>
								</div>
								<p class="font-semibold text-gray-900">{formatCurrency(account.balance)}</p>
							</div>
						</div>
					{/each}
				</div>
			</div>
		</div>
	{/if}
</div>
