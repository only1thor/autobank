<script lang="ts">
	import type { Transaction } from '$lib/api/types';
	import { ArrowDownLeft, ArrowUpRight } from 'lucide-svelte';

	interface Props {
		transactions: Transaction[];
	}

	let { transactions }: Props = $props();

	function formatCurrency(amount: number, currency: string): string {
		return new Intl.NumberFormat('nb-NO', {
			style: 'currency',
			currency
		}).format(amount);
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString('nb-NO', {
			day: 'numeric',
			month: 'short',
			year: 'numeric'
		});
	}
</script>

<div class="card overflow-hidden">
	<table class="table">
		<thead>
			<tr>
				<th>Date</th>
				<th>Description</th>
				<th>Status</th>
				<th class="text-right">Amount</th>
			</tr>
		</thead>
		<tbody>
			{#each transactions as tx (tx.id)}
				<tr>
					<td class="whitespace-nowrap">{formatDate(tx.date)}</td>
					<td>
						<div class="flex items-center gap-2">
							{#if tx.amount < 0}
								<ArrowUpRight class="h-4 w-4 text-red-500 flex-shrink-0" />
							{:else}
								<ArrowDownLeft class="h-4 w-4 text-green-500 flex-shrink-0" />
							{/if}
							<div>
								<div class="font-medium text-gray-100">
									{tx.cleanedDescription || tx.description || 'Unknown'}
								</div>
								{#if tx.kidOrMessage}
									<div class="text-xs text-gray-400">{tx.kidOrMessage}</div>
								{/if}
							</div>
						</div>
					</td>
					<td>
						<span
							class="badge {tx.bookingStatus === 'BOOKED'
								? 'badge-success'
								: 'bg-yellow-100 text-yellow-800'}"
						>
							{tx.bookingStatus === 'BOOKED' ? 'Settled' : 'Pending'}
						</span>
					</td>
					<td class="text-right whitespace-nowrap font-medium {tx.amount < 0 ? 'text-red-400' : 'text-green-400'}">
						{formatCurrency(tx.amount, tx.currencyCode)}
					</td>
				</tr>
			{:else}
				<tr>
					<td colspan="4" class="text-center py-8 text-gray-400">
						No transactions found
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
