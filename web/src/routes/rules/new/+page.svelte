<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api';
	import type { Account, CreateRuleRequest } from '$lib/api/types';
	import RuleBuilder from '$lib/components/rules/RuleBuilder.svelte';
	import { ArrowLeft, RefreshCw, AlertCircle } from 'lucide-svelte';

	let accounts = $state<Account[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let submitting = $state(false);

	onMount(async () => {
		try {
			const data = await api.getAccounts();
			accounts = data.accounts;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load accounts';
		} finally {
			loading = false;
		}
	});

	async function handleSubmit(data: CreateRuleRequest) {
		submitting = true;
		try {
			await api.createRule(data);
			goto('/rules');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create rule';
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
			<h1 class="text-2xl font-bold text-gray-900">Create Rule</h1>
			<p class="text-gray-500">Set up a new automation rule</p>
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
		</div>
	{:else}
		<RuleBuilder {accounts} onsubmit={handleSubmit} oncancel={handleCancel} {submitting} />
	{/if}
</div>
