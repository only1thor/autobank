<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { RuleExecution } from '$lib/api/types';
	import ExecutionList from '$lib/components/executions/ExecutionList.svelte';
	import { RefreshCw, AlertCircle } from 'lucide-svelte';

	let executions = $state<RuleExecution[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let limit = $state(50);

	onMount(async () => {
		await loadExecutions();
	});

	async function loadExecutions() {
		loading = true;
		error = null;
		try {
			executions = await api.getExecutions(limit);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load executions';
		} finally {
			loading = false;
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-900">Executions</h1>
			<p class="text-gray-500">History of rule executions</p>
		</div>
		<div class="flex items-center gap-3">
			<select bind:value={limit} onchange={loadExecutions} class="input w-auto">
				<option value={25}>Last 25</option>
				<option value={50}>Last 50</option>
				<option value={100}>Last 100</option>
			</select>
			<button class="btn btn-secondary" onclick={loadExecutions} disabled={loading}>
				<RefreshCw class="h-4 w-4 mr-2 {loading ? 'animate-spin' : ''}" />
				Refresh
			</button>
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
			<button class="btn btn-secondary mt-4" onclick={loadExecutions}>Retry</button>
		</div>
	{:else}
		<ExecutionList {executions} />
	{/if}
</div>
