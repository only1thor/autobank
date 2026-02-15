<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { Rule } from '$lib/api/types';
	import RuleList from '$lib/components/rules/RuleList.svelte';
	import ConfirmDialog from '$lib/components/common/ConfirmDialog.svelte';
	import { RefreshCw, AlertCircle, Plus } from 'lucide-svelte';
	import { goto } from '$app/navigation';

	let rules = $state<Rule[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let deleteId = $state<string | null>(null);
	let deleting = $state(false);

	onMount(async () => {
		await loadRules();
	});

	async function loadRules() {
		loading = true;
		error = null;
		try {
			rules = await api.getRules();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load rules';
		} finally {
			loading = false;
		}
	}

	function handleUpdate(rule: Rule) {
		rules = rules.map((r) => (r.id === rule.id ? rule : r));
	}

	function handleEdit(rule: Rule) {
		goto(`/rules/${rule.id}`);
	}

	async function confirmDelete() {
		if (!deleteId) return;
		deleting = true;
		try {
			await api.deleteRule(deleteId);
			rules = rules.filter((r) => r.id !== deleteId);
		} catch (e) {
			console.error('Failed to delete rule:', e);
		} finally {
			deleting = false;
			deleteId = null;
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-900">Rules</h1>
			<p class="text-gray-500">Automation rules for your transactions</p>
		</div>
		<a href="/rules/new" class="btn btn-primary">
			<Plus class="h-4 w-4 mr-2" />
			New Rule
		</a>
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
			<button class="btn btn-secondary mt-4" onclick={loadRules}>Retry</button>
		</div>
	{:else}
		<RuleList {rules} onupdate={handleUpdate} ondelete={(id) => (deleteId = id)} onedit={handleEdit} />
	{/if}
</div>

<ConfirmDialog
	open={deleteId !== null}
	title="Delete Rule"
	message="Are you sure you want to delete this rule? This action cannot be undone."
	confirmText={deleting ? 'Deleting...' : 'Delete'}
	variant="danger"
	onconfirm={confirmDelete}
	oncancel={() => (deleteId = null)}
/>
