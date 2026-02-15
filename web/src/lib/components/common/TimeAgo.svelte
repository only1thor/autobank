<script lang="ts">
	interface Props {
		timestamp: number;
	}

	let { timestamp }: Props = $props();

	function formatTimeAgo(ts: number): string {
		const now = Date.now() / 1000;
		const diff = now - ts;

		if (diff < 60) return 'just now';
		if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
		if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
		if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;

		return new Date(ts * 1000).toLocaleDateString();
	}

	let display = $derived(formatTimeAgo(timestamp));
	let fullDate = $derived(new Date(timestamp * 1000).toLocaleString());
</script>

<time datetime={new Date(timestamp * 1000).toISOString()} title={fullDate}>
	{display}
</time>
