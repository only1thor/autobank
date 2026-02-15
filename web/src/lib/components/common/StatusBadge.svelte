<script lang="ts">
	interface Props {
		status: 'success' | 'error' | 'warning' | 'info' | 'pending';
		text?: string;
	}

	let { status, text }: Props = $props();

	const statusClasses: Record<string, string> = {
		success: 'badge-success',
		error: 'badge-error',
		warning: 'badge-warning',
		info: 'badge-info',
		pending: 'bg-gray-100 text-gray-800'
	};

	const defaultText: Record<string, string> = {
		success: 'Success',
		error: 'Error',
		warning: 'Warning',
		info: 'Info',
		pending: 'Pending'
	};
</script>

<span class="badge {statusClasses[status]}">
	{text ?? defaultText[status]}
</span>
