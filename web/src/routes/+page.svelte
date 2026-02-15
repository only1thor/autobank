<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { Account, RuleExecution, SystemStatus, Rule } from '$lib/api/types';
	import AccountCard from '$lib/components/accounts/AccountCard.svelte';
	import ExecutionList from '$lib/components/executions/ExecutionList.svelte';
	import { RefreshCw, Sparkles, CheckCircle, AlertCircle } from 'lucide-svelte';

	let accounts = $state<Account[]>([]);
	let executions = $state<RuleExecution[]>([]);
	let status = $state<SystemStatus | null>(null);
	let rules = $state<Rule[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let polling = $state(false);

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		try {
			const [accountData, execData, statusData, rulesData] = await Promise.all([
				api.getAccounts(),
				api.getExecutions(5),
				api.getSystemStatus(),
				api.getRules()
			]);
			accounts = accountData.accounts;
			executions = execData;
			status = statusData;
			rules = rulesData;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
		} finally {
			loading = false;
		}
	}

	async function triggerPoll() {
		polling = true;
		try {
			await api.triggerPoll();
			await loadData();
		} catch (e) {
			console.error('Poll failed:', e);
		} finally {
			polling = false;
		}
	}

	function formatCurrency(amount: number): string {
		return new Intl.NumberFormat('nb-NO', {
			style: 'currency',
			currency: 'NOK'
		}).format(amount);
	}

	const totalBalance = $derived(accounts.reduce((sum, a) => sum + a.balance, 0));
	const enabledRules = $derived(rules.filter((r) => r.enabled).length);
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-900">Dashboard</h1>
			<p class="text-gray-500">Overview of your accounts and automation</p>
		</div>
		<button class="btn btn-primary" onclick={triggerPoll} disabled={polling}>
			<RefreshCw class="h-4 w-4 mr-2 {polling ? 'animate-spin' : ''}" />
			{polling ? 'Polling...' : 'Poll Now'}
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<RefreshCw class="h-8 w-8 animate-spin text-primary-600" />
		</div>
	{:else if error}
		<div class="card p-6 bg-red-50 border-red-200">
			<div class="flex items-center gap-3 text-red-700">
				<AlertCircle class="h-5 w-5" />
				<p>{error}</p>
			</div>
			<button class="btn btn-secondary mt-4" onclick={loadData}>Retry</button>
		</div>
	{:else}
		<!-- Stats cards -->
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
			<div class="card p-4">
				<div class="text-sm font-medium text-gray-500">Total Balance</div>
				<div class="mt-1 text-2xl font-semibold text-gray-900">
					{formatCurrency(totalBalance)}
				</div>
			</div>
			<div class="card p-4">
				<div class="text-sm font-medium text-gray-500">Accounts</div>
				<div class="mt-1 text-2xl font-semibold text-gray-900">{accounts.length}</div>
			</div>
			<div class="card p-4">
				<div class="flex items-center gap-2">
					<Sparkles class="h-4 w-4 text-primary-600" />
					<div class="text-sm font-medium text-gray-500">Active Rules</div>
				</div>
				<div class="mt-1 text-2xl font-semibold text-gray-900">
					{enabledRules} / {rules.length}
				</div>
			</div>
			<div class="card p-4">
				<div class="flex items-center gap-2">
					{#if status?.scheduler_enabled}
						<CheckCircle class="h-4 w-4 text-green-600" />
					{:else}
						<AlertCircle class="h-4 w-4 text-yellow-600" />
					{/if}
					<div class="text-sm font-medium text-gray-500">Scheduler</div>
				</div>
				<div class="mt-1 text-2xl font-semibold {status?.scheduler_enabled ? 'text-green-600' : 'text-yellow-600'}">
					{status?.scheduler_enabled ? 'Active' : 'Paused'}
				</div>
			</div>
		</div>

		<!-- Accounts preview -->
		<div>
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-lg font-semibold text-gray-900">Accounts</h2>
				<a href="/accounts" class="text-sm text-primary-600 hover:text-primary-700">View all</a>
			</div>
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
				{#each accounts.slice(0, 3) as account (account.key)}
					<AccountCard {account} onclick={() => (window.location.href = `/accounts/${account.key}`)} />
				{/each}
			</div>
		</div>

		<!-- Recent executions -->
		<div>
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-lg font-semibold text-gray-900">Recent Executions</h2>
				<a href="/executions" class="text-sm text-primary-600 hover:text-primary-700">View all</a>
			</div>
			<ExecutionList {executions} />
		</div>
	{/if}
</div>
