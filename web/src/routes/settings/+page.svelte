<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { SystemStatus } from '$lib/api/types';
	import TimeAgo from '$lib/components/common/TimeAgo.svelte';
	import { RefreshCw, AlertCircle, Play, Pause } from 'lucide-svelte';

	let status = $state<SystemStatus | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let toggling = $state(false);

	onMount(async () => {
		await loadStatus();
	});

	async function loadStatus() {
		loading = true;
		error = null;
		try {
			status = await api.getSystemStatus();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load status';
		} finally {
			loading = false;
		}
	}

	async function toggleScheduler() {
		if (!status) return;
		toggling = true;
		try {
			if (status.scheduler_enabled) {
				await api.disableScheduler();
			} else {
				await api.enableScheduler();
			}
			await loadStatus();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to toggle scheduler';
		} finally {
			toggling = false;
		}
	}

	async function triggerPoll() {
		try {
			await api.triggerPoll();
			await loadStatus();
		} catch (e) {
			console.error('Poll failed:', e);
		}
	}
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-2xl font-bold text-gray-100">Settings</h1>
		<p class="text-gray-400">System configuration and status</p>
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
			<button class="btn btn-secondary mt-4" onclick={loadStatus}>Retry</button>
		</div>
	{:else if status}
		<!-- Scheduler -->
		<div class="card p-6">
			<h2 class="text-lg font-semibold text-gray-100 mb-4">Scheduler</h2>
			
			<div class="flex items-center justify-between">
				<div>
					<div class="flex items-center gap-2">
						<span class="text-sm font-medium text-gray-300">Status:</span>
						<span class="badge {status.scheduler_enabled ? 'badge-success' : 'bg-gray-700 text-gray-400'}">
							{status.scheduler_enabled ? 'Running' : 'Paused'}
						</span>
					</div>
					{#if status.last_poll}
						<p class="text-sm text-gray-400 mt-1">
							Last poll: <TimeAgo timestamp={status.last_poll} />
						</p>
					{/if}
				</div>
				<div class="flex items-center gap-2">
					<button class="btn btn-secondary" onclick={triggerPoll}>
						<RefreshCw class="h-4 w-4 mr-2" />
						Poll Now
					</button>
					<button
						class="btn {status.scheduler_enabled ? 'btn-secondary' : 'btn-primary'}"
						onclick={toggleScheduler}
						disabled={toggling}
					>
						{#if status.scheduler_enabled}
							<Pause class="h-4 w-4 mr-2" />
							Pause
						{:else}
							<Play class="h-4 w-4 mr-2" />
							Start
						{/if}
					</button>
				</div>
			</div>
		</div>

		<!-- Stats -->
		<div class="card p-6">
			<h2 class="text-lg font-semibold text-gray-100 mb-4">Statistics</h2>
			
			<div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
				<div>
					<div class="text-xs text-gray-400 uppercase">Total Rules</div>
					<div class="text-2xl font-semibold text-gray-100">{status.total_rules}</div>
				</div>
				<div>
					<div class="text-xs text-gray-400 uppercase">Enabled Rules</div>
					<div class="text-2xl font-semibold text-gray-100">{status.enabled_rules}</div>
				</div>
				<div>
					<div class="text-xs text-gray-400 uppercase">Total Executions</div>
					<div class="text-2xl font-semibold text-gray-100">{status.total_executions}</div>
				</div>
				<div>
					<div class="text-xs text-gray-400 uppercase">System Status</div>
					<div class="text-2xl font-semibold text-green-400">{status.status}</div>
				</div>
			</div>
		</div>

		<!-- API Info -->
		<div class="card p-6">
			<h2 class="text-lg font-semibold text-gray-100 mb-4">API</h2>
			<p class="text-sm text-gray-400">
				The backend API is available at <code class="bg-gray-700 px-1 rounded">/api</code>
			</p>
			<p class="text-sm text-gray-400 mt-2">
				Make sure the autobank-server is running on port 3000 for the frontend to connect.
			</p>
		</div>
	{/if}
</div>
