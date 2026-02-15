<script lang="ts">
	import type { Account, Condition, Action, AmountSpec, AccountRef, CreateRuleRequest } from '$lib/api/types';
	import AccountSelect from '$lib/components/accounts/AccountSelect.svelte';
	import { Plus, Trash2 } from 'lucide-svelte';

	interface Props {
		accounts: Account[];
		initialData?: Partial<CreateRuleRequest>;
		onsubmit: (data: CreateRuleRequest) => void;
		oncancel: () => void;
		submitting?: boolean;
	}

	let { accounts, initialData, onsubmit, oncancel, submitting = false }: Props = $props();

	let name = $state(initialData?.name ?? '');
	let description = $state(initialData?.description ?? '');
	let triggerAccountKey = $state(initialData?.trigger_account_key ?? '');
	let conditions = $state<Condition[]>(initialData?.conditions ?? []);
	let actions = $state<Action[]>(initialData?.actions ?? []);

	// Condition form state
	let newConditionType = $state<string>('description_matches');
	let conditionPattern = $state('');
	let conditionCaseInsensitive = $state(true);
	let conditionValue = $state(0);
	let conditionMin = $state(0);
	let conditionMax = $state(0);
	let conditionTypeCode = $state('');

	// Action form state
	let actionFromType = $state<'trigger_account' | 'by_key'>('trigger_account');
	let actionFromKey = $state('');
	let actionToType = $state<'trigger_account' | 'by_key'>('by_key');
	let actionToKey = $state('');
	let actionAmountType = $state<'fixed' | 'transaction_amount' | 'transaction_amount_abs' | 'percentage'>('transaction_amount_abs');
	let actionAmountValue = $state(0);
	let actionMessage = $state('');

	function addCondition() {
		let condition: Condition;
		switch (newConditionType) {
			case 'description_matches':
				condition = { type: 'description_matches', pattern: conditionPattern, case_insensitive: conditionCaseInsensitive };
				break;
			case 'amount_greater_than':
				condition = { type: 'amount_greater_than', value: conditionValue };
				break;
			case 'amount_less_than':
				condition = { type: 'amount_less_than', value: conditionValue };
				break;
			case 'amount_between':
				condition = { type: 'amount_between', min: conditionMin, max: conditionMax };
				break;
			case 'transaction_type':
				condition = { type: 'transaction_type', type_code: conditionTypeCode };
				break;
			case 'is_settled':
				condition = { type: 'is_settled' };
				break;
			default:
				return;
		}
		conditions = [...conditions, condition];
		// Reset form
		conditionPattern = '';
		conditionValue = 0;
	}

	function removeCondition(index: number) {
		conditions = conditions.filter((_, i) => i !== index);
	}

	function addAction() {
		const fromAccount: AccountRef =
			actionFromType === 'trigger_account' ? { type: 'trigger_account' } : { type: 'by_key', key: actionFromKey };
		const toAccount: AccountRef =
			actionToType === 'trigger_account' ? { type: 'trigger_account' } : { type: 'by_key', key: actionToKey };

		let amount: AmountSpec;
		switch (actionAmountType) {
			case 'fixed':
				amount = { type: 'fixed', value: actionAmountValue };
				break;
			case 'transaction_amount':
				amount = { type: 'transaction_amount' };
				break;
			case 'transaction_amount_abs':
				amount = { type: 'transaction_amount_abs' };
				break;
			case 'percentage':
				amount = { type: 'percentage', of_transaction: actionAmountValue };
				break;
		}

		const action: Action = {
			type: 'transfer',
			from_account: fromAccount,
			to_account: toAccount,
			amount,
			message: actionMessage || undefined
		};
		actions = [...actions, action];
	}

	function removeAction(index: number) {
		actions = actions.filter((_, i) => i !== index);
	}

	function formatCondition(c: Condition): string {
		switch (c.type) {
			case 'description_matches':
				return `Description matches "${c.pattern}"${c.case_insensitive ? ' (case-insensitive)' : ''}`;
			case 'amount_greater_than':
				return `Amount > ${c.value}`;
			case 'amount_less_than':
				return `Amount < ${c.value}`;
			case 'amount_between':
				return `Amount between ${c.min} and ${c.max}`;
			case 'transaction_type':
				return `Transaction type = "${c.type_code}"`;
			case 'is_settled':
				return 'Transaction is settled';
			default:
				return c.type;
		}
	}

	function formatAccountRef(ref: AccountRef): string {
		switch (ref.type) {
			case 'trigger_account':
				return 'Trigger account';
			case 'by_key':
				const acc = accounts.find((a) => a.key === ref.key);
				return acc ? acc.name : ref.key;
			case 'by_number':
				return ref.number;
		}
	}

	function formatAmount(spec: AmountSpec): string {
		switch (spec.type) {
			case 'fixed':
				return `${spec.value} NOK`;
			case 'transaction_amount':
				return 'Transaction amount';
			case 'transaction_amount_abs':
				return 'Transaction amount (absolute)';
			case 'percentage':
				return `${spec.of_transaction}% of transaction`;
			default:
				return spec.type;
		}
	}

	function handleSubmit() {
		const data: CreateRuleRequest = {
			name,
			description: description || undefined,
			trigger_account_key: triggerAccountKey,
			conditions,
			actions
		};
		onsubmit(data);
	}

	const isValid = $derived(name && triggerAccountKey && conditions.length > 0 && actions.length > 0);
</script>

<form class="space-y-6" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
	<!-- Basic info -->
	<div class="card p-6 space-y-4">
		<h2 class="text-lg font-semibold text-gray-900">Basic Information</h2>
		
		<div>
			<label for="name" class="label">Rule Name *</label>
			<input type="text" id="name" bind:value={name} class="input" placeholder="e.g., Netflix auto-transfer" required />
		</div>

		<div>
			<label for="description" class="label">Description</label>
			<textarea id="description" bind:value={description} class="input" rows="2" placeholder="What does this rule do?"></textarea>
		</div>

		<div>
			<label class="label">Trigger Account *</label>
			<p class="text-xs text-gray-500 mb-2">Which account to monitor for transactions</p>
			<AccountSelect {accounts} selected={triggerAccountKey} onselect={(key) => (triggerAccountKey = key)} placeholder="Select trigger account" />
		</div>
	</div>

	<!-- Conditions -->
	<div class="card p-6 space-y-4">
		<h2 class="text-lg font-semibold text-gray-900">Conditions</h2>
		<p class="text-sm text-gray-500">When should this rule trigger? All conditions must match.</p>

		{#if conditions.length > 0}
			<div class="space-y-2">
				{#each conditions as condition, i}
					<div class="flex items-center justify-between bg-gray-50 rounded px-3 py-2">
						<span class="text-sm">{formatCondition(condition)}</span>
						<button type="button" class="btn btn-ghost p-1 text-red-600" onclick={() => removeCondition(i)}>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>
				{/each}
			</div>
		{/if}

		<div class="border-t pt-4 space-y-3">
			<div>
				<label class="label">Add Condition</label>
				<select bind:value={newConditionType} class="input">
					<option value="description_matches">Description matches</option>
					<option value="amount_greater_than">Amount greater than</option>
					<option value="amount_less_than">Amount less than</option>
					<option value="amount_between">Amount between</option>
					<option value="transaction_type">Transaction type</option>
					<option value="is_settled">Is settled</option>
				</select>
			</div>

			{#if newConditionType === 'description_matches'}
				<div>
					<label class="label">Pattern (regex)</label>
					<input type="text" bind:value={conditionPattern} class="input" placeholder="e.g., netflix|spotify" />
				</div>
				<label class="flex items-center gap-2 text-sm">
					<input type="checkbox" bind:checked={conditionCaseInsensitive} class="rounded border-gray-300" />
					Case insensitive
				</label>
			{:else if newConditionType === 'amount_greater_than' || newConditionType === 'amount_less_than'}
				<div>
					<label class="label">Value</label>
					<input type="number" step="0.01" bind:value={conditionValue} class="input" />
				</div>
			{:else if newConditionType === 'amount_between'}
				<div class="grid grid-cols-2 gap-4">
					<div>
						<label class="label">Min</label>
						<input type="number" step="0.01" bind:value={conditionMin} class="input" />
					</div>
					<div>
						<label class="label">Max</label>
						<input type="number" step="0.01" bind:value={conditionMax} class="input" />
					</div>
				</div>
			{:else if newConditionType === 'transaction_type'}
				<div>
					<label class="label">Type Code</label>
					<input type="text" bind:value={conditionTypeCode} class="input" placeholder="e.g., PURCHASE" />
				</div>
			{/if}

			<button type="button" class="btn btn-secondary" onclick={addCondition}>
				<Plus class="h-4 w-4 mr-2" />
				Add Condition
			</button>
		</div>
	</div>

	<!-- Actions -->
	<div class="card p-6 space-y-4">
		<h2 class="text-lg font-semibold text-gray-900">Actions</h2>
		<p class="text-sm text-gray-500">What should happen when conditions match?</p>

		{#if actions.length > 0}
			<div class="space-y-2">
				{#each actions as action, i}
					<div class="flex items-center justify-between bg-gray-50 rounded px-3 py-2">
						<span class="text-sm">
							Transfer {formatAmount(action.amount)} from {formatAccountRef(action.from_account)} to {formatAccountRef(action.to_account)}
							{#if action.message}
								<span class="text-gray-500"> - "{action.message}"</span>
							{/if}
						</span>
						<button type="button" class="btn btn-ghost p-1 text-red-600" onclick={() => removeAction(i)}>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>
				{/each}
			</div>
		{/if}

		<div class="border-t pt-4 space-y-3">
			<div class="grid grid-cols-2 gap-4">
				<div>
					<label class="label">From Account</label>
					<select bind:value={actionFromType} class="input">
						<option value="trigger_account">Trigger Account</option>
						<option value="by_key">Select Account</option>
					</select>
					{#if actionFromType === 'by_key'}
						<div class="mt-2">
							<AccountSelect {accounts} selected={actionFromKey} onselect={(key) => (actionFromKey = key)} />
						</div>
					{/if}
				</div>
				<div>
					<label class="label">To Account</label>
					<select bind:value={actionToType} class="input">
						<option value="trigger_account">Trigger Account</option>
						<option value="by_key">Select Account</option>
					</select>
					{#if actionToType === 'by_key'}
						<div class="mt-2">
							<AccountSelect {accounts} selected={actionToKey} onselect={(key) => (actionToKey = key)} />
						</div>
					{/if}
				</div>
			</div>

			<div>
				<label class="label">Amount</label>
				<select bind:value={actionAmountType} class="input">
					<option value="transaction_amount_abs">Transaction Amount (absolute)</option>
					<option value="transaction_amount">Transaction Amount</option>
					<option value="fixed">Fixed Amount</option>
					<option value="percentage">Percentage of Transaction</option>
				</select>
				{#if actionAmountType === 'fixed'}
					<input type="number" step="0.01" bind:value={actionAmountValue} class="input mt-2" placeholder="Amount in NOK" />
				{:else if actionAmountType === 'percentage'}
					<input type="number" step="1" bind:value={actionAmountValue} class="input mt-2" placeholder="Percentage (e.g., 10)" />
				{/if}
			</div>

			<div>
				<label class="label">Message (optional)</label>
				<input type="text" bind:value={actionMessage} class="input" placeholder="Transfer message" maxlength="40" />
			</div>

			<button type="button" class="btn btn-secondary" onclick={addAction}>
				<Plus class="h-4 w-4 mr-2" />
				Add Action
			</button>
		</div>
	</div>

	<!-- Submit -->
	<div class="flex justify-end gap-3">
		<button type="button" class="btn btn-secondary" onclick={oncancel}>Cancel</button>
		<button type="submit" class="btn btn-primary" disabled={!isValid || submitting}>
			{submitting ? 'Saving...' : 'Save Rule'}
		</button>
	</div>
</form>
