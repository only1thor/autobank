<script lang="ts">
	import type { AuditEntry } from '$lib/api/types';
	import TimeAgo from '../common/TimeAgo.svelte';
	import { ChevronDown, ChevronRight } from 'lucide-svelte';

	interface Props {
		entries: AuditEntry[];
	}

	let { entries }: Props = $props();
	let expandedId = $state<string | null>(null);

	function getEventTypeColor(eventType: string): string {
		if (eventType.includes('failed') || eventType.includes('error')) return 'text-red-600';
		if (eventType.includes('success') || eventType.includes('completed')) return 'text-green-600';
		if (eventType.includes('started') || eventType.includes('initiated')) return 'text-blue-600';
		return 'text-gray-600';
	}

	function formatEventType(eventType: string): string {
		return eventType.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
	}
</script>

<div class="card overflow-hidden">
	<table class="table">
		<thead>
			<tr>
				<th class="w-8"></th>
				<th>Time</th>
				<th>Event</th>
				<th>Actor</th>
				<th>Resource</th>
			</tr>
		</thead>
		<tbody>
			{#each entries as entry (entry.id)}
				<tr
					class="cursor-pointer"
					onclick={() => (expandedId = expandedId === entry.id ? null : entry.id)}
				>
					<td class="w-8">
						{#if expandedId === entry.id}
							<ChevronDown class="h-4 w-4 text-gray-400" />
						{:else}
							<ChevronRight class="h-4 w-4 text-gray-400" />
						{/if}
					</td>
					<td class="whitespace-nowrap">
						<TimeAgo timestamp={entry.timestamp} />
					</td>
					<td>
						<span class="font-medium {getEventTypeColor(entry.event_type)}">
							{formatEventType(entry.event_type)}
						</span>
					</td>
					<td class="text-sm text-gray-600">{entry.actor}</td>
					<td class="text-sm text-gray-600">
						{#if entry.resource_type}
							{entry.resource_type}
							{#if entry.resource_id}
								<span class="text-gray-400">({entry.resource_id.slice(0, 8)}...)</span>
							{/if}
						{:else}
							-
						{/if}
					</td>
				</tr>
				{#if expandedId === entry.id}
					<tr>
						<td colspan="5" class="bg-gray-50 p-4">
							<pre class="text-xs overflow-x-auto">{JSON.stringify(entry.details, null, 2)}</pre>
						</td>
					</tr>
				{/if}
			{:else}
				<tr>
					<td colspan="5" class="text-center py-8 text-gray-500">
						No audit entries
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
