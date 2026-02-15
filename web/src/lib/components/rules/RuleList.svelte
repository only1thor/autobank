<script lang="ts">
	import type { Rule } from '$lib/api/types';
	import RuleCard from './RuleCard.svelte';

	interface Props {
		rules: Rule[];
		onupdate: (rule: Rule) => void;
		ondelete: (id: string) => void;
		onedit: (rule: Rule) => void;
	}

	let { rules, onupdate, ondelete, onedit }: Props = $props();
</script>

<div class="space-y-4">
	{#each rules as rule (rule.id)}
		<RuleCard {rule} {onupdate} {ondelete} {onedit} />
	{:else}
		<div class="card p-8 text-center text-gray-400">
			<p>No rules yet. Create your first automation rule to get started.</p>
		</div>
	{/each}
</div>
