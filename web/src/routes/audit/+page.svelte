<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { AuditEntry } from '$lib/api/types';
	import AuditLog from '$lib/components/audit/AuditLog.svelte';
	import { RefreshCw, AlertCircle } from 'lucide-svelte';

	let entries = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let limit = $state(50);

	onMount(async () => {
		await loadAudit();
	});

	async function loadAudit() {
		loading = true;
		error = null;
		try {
			entries = await api.getAuditLog(limit);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load audit log';
		} finally {
			loading = false;
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-100">Audit Log</h1>
			<p class="text-gray-400">System events and activity</p>
		</div>
		<div class="flex items-center gap-3">
			<select bind:value={limit} onchange={loadAudit} class="input w-auto">
				<option value={25}>Last 25</option>
				<option value={50}>Last 50</option>
				<option value={100}>Last 100</option>
			</select>
			<button class="btn btn-secondary" onclick={loadAudit} disabled={loading}>
				<RefreshCw class="h-4 w-4 mr-2 {loading ? 'animate-spin' : ''}" />
				Refresh
			</button>
		</div>
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
			<button class="btn btn-secondary mt-4" onclick={loadAudit}>Retry</button>
		</div>
	{:else}
		<AuditLog {entries} />
	{/if}
</div>
