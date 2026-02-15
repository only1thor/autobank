<script lang="ts">
	import type { Account } from '$lib/api/types';
	import { ChevronDown } from 'lucide-svelte';

	interface Props {
		accounts: Account[];
		selected?: string;
		placeholder?: string;
		onselect: (key: string) => void;
	}

	let { accounts, selected, placeholder = 'Select account', onselect }: Props = $props();

	let open = $state(false);
	const selectedAccount = $derived(accounts.find((a) => a.key === selected));

	function formatCurrency(amount: number, currency: string): string {
		return new Intl.NumberFormat('nb-NO', {
			style: 'currency',
			currency
		}).format(amount);
	}

	function handleSelect(key: string) {
		onselect(key);
		open = false;
	}
</script>

<div class="relative">
	<button
		type="button"
		class="input flex items-center justify-between bg-white"
		onclick={() => (open = !open)}
	>
		{#if selectedAccount}
			<span class="flex items-center gap-2">
				<span>{selectedAccount.name}</span>
				<span class="text-gray-500">
					({formatCurrency(selectedAccount.balance, selectedAccount.currencyCode)})
				</span>
			</span>
		{:else}
			<span class="text-gray-400">{placeholder}</span>
		{/if}
		<ChevronDown class="h-4 w-4 text-gray-400" />
	</button>

	{#if open}
		<div class="absolute z-10 mt-1 w-full rounded-md bg-white shadow-lg border border-gray-200">
			<ul class="max-h-60 overflow-auto py-1">
				{#each accounts as account (account.key)}
					<li>
						<button
							type="button"
							class="w-full px-3 py-2 text-left hover:bg-gray-100 {selected === account.key
								? 'bg-primary-50'
								: ''}"
							onclick={() => handleSelect(account.key)}
						>
							<div class="font-medium">{account.name}</div>
							<div class="text-sm text-gray-500">
								{account.accountNumber} -
								{formatCurrency(account.balance, account.currencyCode)}
							</div>
						</button>
					</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>

<svelte:window
	onclick={(e) => {
		const target = e.target as HTMLElement;
		if (!target.closest('.relative')) open = false;
	}}
/>
