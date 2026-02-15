// Types matching the Rust backend models

// Account types (from sb1-api)
export interface AccountData {
	accounts: Account[];
	errors: unknown[];
}

export interface Account {
	key: string;
	accountNumber: string;
	iban: string;
	name: string;
	description: string;
	balance: number;
	availableBalance: number;
	currencyCode: string;
	owner?: Owner;
	productType: string;
	type: string;
	productId?: string;
	descriptionCode?: string;
	accountProperties: AccountProperties;
	creditCardCreditLimit?: number;
	creditCardAccountID?: string;
}

export interface Owner {
	name: string;
	firstName: string;
	lastName: string;
	age: number;
	customerKey: string;
	ssnKey: string;
}

export interface AccountProperties {
	isTransferFromEnabled: boolean;
	isTransferToEnabled: boolean;
	isPaymentFromEnabled: boolean;
	isAllowedInAvtaleGiro: boolean;
	hasAccess: boolean;
	isBalancePreferred: boolean;
	isFlexiLoan: boolean;
	isCodebitorLoan: boolean;
	isSecurityBalance: boolean;
	isAksjesparekonto: boolean;
	isSavingsAccount: boolean;
	isBonusAccount: boolean;
	userHasRightOfDisposal: boolean;
	userHasRightOfAccess: boolean;
	isOwned: boolean;
	isWithdrawalsAllowed: boolean;
	isBlocked: boolean;
	isHidden: boolean;
	isBalanceUpdatedImmediatelyOnTransferTo: boolean;
	isDefaultPaymentAccount: boolean;
}

// Transaction types
export interface TransactionResponse {
	transactions: Transaction[];
	errors: unknown[];
}

export interface Transaction {
	id: string;
	nonUniqueId: string;
	description?: string;
	cleanedDescription?: string;
	accountNumber: AccountNumber;
	amount: number;
	date: number;
	interestDate?: number;
	typeCode: string;
	typeText: string;
	currencyCode: string;
	canShowDetails: boolean;
	source: string;
	isConfidential: boolean;
	bookingStatus: string;
	accountName: string;
	accountKey: string;
	accountCurrency: string;
	isFromCurrencyAccount: boolean;
	classificationInput: ClassificationInput;
	remoteAccountNumber?: string;
	remoteAccountName?: string;
	kidOrMessage?: string;
}

export interface AccountNumber {
	value: string;
	formatted: string;
	unformatted: string;
}

export interface ClassificationInput {
	id: string;
	amount: number;
	type: string;
	text?: string;
	date: number;
}

// Rule types
export interface Rule {
	id: string;
	name: string;
	description?: string;
	enabled: boolean;
	trigger_account_key: string;
	conditions: Condition[];
	actions: Action[];
	created_at: number;
	updated_at: number;
}

export interface CreateRuleRequest {
	name: string;
	description?: string;
	trigger_account_key: string;
	conditions: Condition[];
	actions: Action[];
}

export interface UpdateRuleRequest {
	name?: string;
	description?: string;
	trigger_account_key?: string;
	conditions?: Condition[];
	actions?: Action[];
}

// Condition types (discriminated union)
export type Condition =
	| { type: 'description_matches'; pattern: string; case_insensitive?: boolean }
	| { type: 'amount_greater_than'; value: number }
	| { type: 'amount_less_than'; value: number }
	| { type: 'amount_between'; min: number; max: number }
	| { type: 'amount_equals'; value: number; tolerance?: number }
	| { type: 'transaction_type'; type_code: string }
	| { type: 'is_settled' }
	| { type: 'and'; conditions: Condition[] }
	| { type: 'or'; conditions: Condition[] }
	| { type: 'not'; condition: Condition };

// Action types
export type Action = {
	type: 'transfer';
	from_account: AccountRef;
	to_account: AccountRef;
	amount: AmountSpec;
	message?: string;
};

// Account reference types
export type AccountRef =
	| { type: 'by_key'; key: string }
	| { type: 'by_number'; number: string }
	| { type: 'trigger_account' };

// Amount specification types
export type AmountSpec =
	| { type: 'fixed'; value: number }
	| { type: 'transaction_amount' }
	| { type: 'transaction_amount_abs' }
	| { type: 'percentage'; of_transaction: number }
	| { type: 'min'; specs: AmountSpec[] }
	| { type: 'max'; specs: AmountSpec[] };

// Execution types
export interface RuleExecution {
	id: string;
	rule_id: string;
	transaction_id: string;
	transfer_payment_id?: string;
	amount: number;
	from_account: string;
	to_account: string;
	status: string;
	error_message?: string;
	executed_at: number;
}

// Audit types
export interface AuditEntry {
	id: string;
	timestamp: number;
	event_type: string;
	actor: string;
	resource_type?: string;
	resource_id?: string;
	details: unknown;
	ip_address?: string;
	user_agent?: string;
}

// System status
export interface SystemStatus {
	status: string;
	scheduler_enabled: boolean;
	last_poll?: number;
	total_rules: number;
	enabled_rules: number;
	total_executions: number;
}

// API Error
export interface ApiError {
	error: string;
}
