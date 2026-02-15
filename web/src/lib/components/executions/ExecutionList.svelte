<script lang="ts">
	import type { RuleExecution } from '$lib/api/types';
	import StatusBadge from '../common/StatusBadge.svelte';
	import TimeAgo from '../common/TimeAgo.svelte';

	interface Props {
		executions: RuleExecution[];
	}

	let { executions }: Props = $props();

	function formatCurrency(amount: number): string {
		return new Intl.NumberFormat('nb-NO', {
			style: 'currency',
			currency: 'NOK'
		}).format(amount);
	}
</script>

<div class="card overflow-hidden">
	<table class="table">
		<thead>
			<tr>
				<th>Time</th>
				<th>Amount</th>
				<th>From</th>
				<th>To</th>
				<th>Status</th>
			</tr>
		</thead>
		<tbody>
			{#each executions as exec (exec.id)}
				<tr>
					<td class="whitespace-nowrap">
						<TimeAgo timestamp={exec.executed_at} />
					</td>
					<td class="font-medium">{formatCurrency(exec.amount)}</td>
					<td class="text-sm text-gray-600">{exec.from_account}</td>
					<td class="text-sm text-gray-600">{exec.to_account}</td>
					<td>
						<StatusBadge
							status={exec.status === 'success' ? 'success' : 'error'}
							text={exec.status}
						/>
						{#if exec.error_message}
							<div class="text-xs text-red-600 mt-1">{exec.error_message}</div>
						{/if}
					</td>
				</tr>
			{:else}
				<tr>
					<td colspan="5" class="text-center py-8 text-gray-500">
						No executions yet
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
