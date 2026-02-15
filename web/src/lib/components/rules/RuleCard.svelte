<script lang="ts">
	import type { Rule } from '$lib/api/types';
	import { api } from '$lib/api';
	import TimeAgo from '../common/TimeAgo.svelte';
	import { Settings, Trash2, Power, PowerOff } from 'lucide-svelte';

	interface Props {
		rule: Rule;
		onupdate: (rule: Rule) => void;
		ondelete: (id: string) => void;
		onedit: (rule: Rule) => void;
	}

	let { rule, onupdate, ondelete, onedit }: Props = $props();

	let toggling = $state(false);

	async function toggleEnabled() {
		toggling = true;
		try {
			const updated = rule.enabled ? await api.disableRule(rule.id) : await api.enableRule(rule.id);
			onupdate(updated);
		} catch (e) {
			console.error('Failed to toggle rule:', e);
		} finally {
			toggling = false;
		}
	}

	function getConditionSummary(rule: Rule): string {
		const conditions = rule.conditions;
		if (conditions.length === 0) return 'No conditions';
		if (conditions.length === 1) {
			const c = conditions[0];
			switch (c.type) {
				case 'description_matches':
					return `Description matches "${c.pattern}"`;
				case 'amount_greater_than':
					return `Amount > ${c.value}`;
				case 'amount_less_than':
					return `Amount < ${c.value}`;
				case 'is_settled':
					return 'Transaction is settled';
				default:
					return c.type.replace(/_/g, ' ');
			}
		}
		return `${conditions.length} conditions`;
	}

	function getActionSummary(rule: Rule): string {
		const actions = rule.actions;
		if (actions.length === 0) return 'No actions';
		const action = actions[0];
		if (action.type === 'transfer') {
			const amount =
				action.amount.type === 'fixed'
					? `${action.amount.value}`
					: action.amount.type === 'transaction_amount_abs'
						? 'transaction amount'
						: action.amount.type;
			return `Transfer ${amount}`;
		}
		return `${actions.length} action(s)`;
	}
</script>

<div class="card p-4">
	<div class="flex items-start justify-between">
		<div class="flex-1">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-100">{rule.name}</h3>
				<span
					class="badge {rule.enabled
						? 'bg-green-100 text-green-800'
						: 'bg-gray-700 text-gray-400'}"
				>
					{rule.enabled ? 'Active' : 'Disabled'}
				</span>
			</div>
			{#if rule.description}
				<p class="mt-1 text-sm text-gray-400">{rule.description}</p>
			{/if}
		</div>

		<div class="flex items-center gap-1">
			<button
				class="btn btn-ghost p-2"
				onclick={toggleEnabled}
				disabled={toggling}
				title={rule.enabled ? 'Disable rule' : 'Enable rule'}
			>
				{#if rule.enabled}
					<PowerOff class="h-4 w-4" />
				{:else}
					<Power class="h-4 w-4" />
				{/if}
			</button>
			<button class="btn btn-ghost p-2" onclick={() => onedit(rule)} title="Edit rule">
				<Settings class="h-4 w-4" />
			</button>
			<button
				class="btn btn-ghost p-2 text-red-400 hover:bg-red-900/30"
				onclick={() => ondelete(rule.id)}
				title="Delete rule"
			>
				<Trash2 class="h-4 w-4" />
			</button>
		</div>
	</div>

	<div class="mt-4 grid gap-2 sm:grid-cols-2 text-sm">
		<div class="rounded bg-gray-800 px-3 py-2">
			<div class="text-xs font-medium text-gray-400 uppercase">When</div>
			<div class="text-gray-100">{getConditionSummary(rule)}</div>
		</div>
		<div class="rounded bg-gray-800 px-3 py-2">
			<div class="text-xs font-medium text-gray-400 uppercase">Then</div>
			<div class="text-gray-100">{getActionSummary(rule)}</div>
		</div>
	</div>

	<div class="mt-3 text-xs text-gray-400">
		Updated <TimeAgo timestamp={rule.updated_at} />
	</div>
</div>
