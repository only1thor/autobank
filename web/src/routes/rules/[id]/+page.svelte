<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api';
	import type { Account, Rule, CreateRuleRequest } from '$lib/api/types';
	import RuleBuilder from '$lib/components/rules/RuleBuilder.svelte';
	import { ArrowLeft, RefreshCw, AlertCircle } from 'lucide-svelte';

	let accounts = $state<Account[]>([]);
	let rule = $state<Rule | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let submitting = $state(false);

	const id = $derived(page.params.id);

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		if (!id) return;
		loading = true;
		error = null;
		try {
			const [accountData, ruleData] = await Promise.all([api.getAccounts(), api.getRule(id)]);
			accounts = accountData.accounts;
			rule = ruleData;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
		} finally {
			loading = false;
		}
	}

	async function handleSubmit(data: CreateRuleRequest) {
		if (!id) return;
		submitting = true;
		try {
			await api.updateRule(id, data);
			goto('/rules');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update rule';
		} finally {
			submitting = false;
		}
	}

	function handleCancel() {
		goto('/rules');
	}
</script>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/rules" class="btn btn-ghost p-2">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<div>
			<h1 class="text-2xl font-bold text-gray-900">Edit Rule</h1>
			{#if rule}
				<p class="text-gray-500">{rule.name}</p>
			{/if}
		</div>
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
	{:else if rule}
		<RuleBuilder
			{accounts}
			initialData={{
				name: rule.name,
				description: rule.description,
				trigger_account_key: rule.trigger_account_key,
				conditions: rule.conditions,
				actions: rule.actions
			}}
			onsubmit={handleSubmit}
			oncancel={handleCancel}
			{submitting}
		/>
	{/if}
</div>
