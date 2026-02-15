<script lang="ts">
	import type { Account } from '$lib/api/types';
	import { CreditCard, Wallet } from 'lucide-svelte';

	interface Props {
		account: Account;
		onclick?: () => void;
	}

	let { account, onclick }: Props = $props();

	function formatCurrency(amount: number, currency: string): string {
		return new Intl.NumberFormat('nb-NO', {
			style: 'currency',
			currency
		}).format(amount);
	}

	const isCreditCard = $derived(account.type === 'CREDITCARD');
</script>

<button
	class="card p-4 w-full text-left hover:border-primary-500 transition-colors cursor-pointer"
	onclick={onclick}
>
	<div class="flex items-start justify-between">
		<div class="flex items-center gap-3">
			<div
				class="flex h-10 w-10 items-center justify-center rounded-full {isCreditCard
					? 'bg-purple-900/50 text-purple-400'
					: 'bg-primary-900/50 text-primary-400'}"
			>
				{#if isCreditCard}
					<CreditCard class="h-5 w-5" />
				{:else}
					<Wallet class="h-5 w-5" />
				{/if}
			</div>
			<div>
				<h3 class="font-medium text-gray-100">{account.name}</h3>
				<p class="text-sm text-gray-400">{account.accountNumber}</p>
			</div>
		</div>
	</div>

	<div class="mt-4">
		<div class="text-2xl font-semibold text-gray-100">
			{formatCurrency(account.balance, account.currencyCode)}
		</div>
		{#if account.availableBalance !== account.balance}
			<div class="text-sm text-gray-400">
				Available: {formatCurrency(account.availableBalance, account.currencyCode)}
			</div>
		{/if}
	</div>

	{#if isCreditCard && account.creditCardCreditLimit}
		<div class="mt-2">
			<div class="flex justify-between text-xs text-gray-400 mb-1">
				<span>Credit used</span>
				<span>{formatCurrency(account.creditCardCreditLimit - account.balance, account.currencyCode)}</span>
			</div>
			<div class="h-2 bg-gray-700 rounded-full overflow-hidden">
				<div
					class="h-full bg-purple-500 rounded-full"
					style="width: {Math.min(100, ((account.creditCardCreditLimit - account.balance) / account.creditCardCreditLimit) * 100)}%"
				></div>
			</div>
		</div>
	{/if}

	<div class="mt-3 flex flex-wrap gap-2">
		{#if account.accountProperties.isSavingsAccount}
			<span class="badge badge-info">Savings</span>
		{/if}
		{#if account.accountProperties.isDefaultPaymentAccount}
			<span class="badge badge-success">Default</span>
		{/if}
		{#if account.accountProperties.isBlocked}
			<span class="badge badge-error">Blocked</span>
		{/if}
	</div>
</button>
