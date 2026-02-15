import type {
	Account,
	AccountData,
	AuditEntry,
	CreateRuleRequest,
	Rule,
	RuleExecution,
	SystemStatus,
	TransactionResponse,
	UpdateRuleRequest
} from './types';

const BASE_URL = '/api';

class ApiClient {
	private async request<T>(path: string, options?: RequestInit): Promise<T> {
		const url = `${BASE_URL}${path}`;
		const response = await fetch(url, {
			...options,
			headers: {
				'Content-Type': 'application/json',
				...options?.headers
			}
		});

		if (!response.ok) {
			const error = await response.json().catch(() => ({ error: 'Unknown error' }));
			throw new Error(error.error || `HTTP ${response.status}`);
		}

		return response.json();
	}

	// Health
	async health(): Promise<{ status: string }> {
		return this.request('/health');
	}

	// Accounts
	async getAccounts(): Promise<AccountData> {
		return this.request('/accounts');
	}

	async getAccount(key: string): Promise<Account> {
		return this.request(`/accounts/${encodeURIComponent(key)}`);
	}

	async getTransactions(accountKey: string): Promise<TransactionResponse> {
		return this.request(`/accounts/${encodeURIComponent(accountKey)}/transactions`);
	}

	// Rules
	async getRules(): Promise<Rule[]> {
		return this.request('/rules');
	}

	async getRule(id: string): Promise<Rule> {
		return this.request(`/rules/${id}`);
	}

	async createRule(rule: CreateRuleRequest): Promise<Rule> {
		return this.request('/rules', {
			method: 'POST',
			body: JSON.stringify(rule)
		});
	}

	async updateRule(id: string, rule: UpdateRuleRequest): Promise<Rule> {
		return this.request(`/rules/${id}`, {
			method: 'PUT',
			body: JSON.stringify(rule)
		});
	}

	async deleteRule(id: string): Promise<void> {
		return this.request(`/rules/${id}`, {
			method: 'DELETE'
		});
	}

	async enableRule(id: string): Promise<Rule> {
		return this.request(`/rules/${id}/enable`, {
			method: 'POST'
		});
	}

	async disableRule(id: string): Promise<Rule> {
		return this.request(`/rules/${id}/disable`, {
			method: 'POST'
		});
	}

	async getRuleExecutions(ruleId: string): Promise<RuleExecution[]> {
		return this.request(`/rules/${ruleId}/executions`);
	}

	// Executions
	async getExecutions(limit?: number): Promise<RuleExecution[]> {
		const params = limit ? `?limit=${limit}` : '';
		return this.request(`/executions${params}`);
	}

	async getExecution(id: string): Promise<RuleExecution> {
		return this.request(`/executions/${id}`);
	}

	// Audit
	async getAuditLog(limit?: number, eventType?: string): Promise<AuditEntry[]> {
		const params = new URLSearchParams();
		if (limit) params.set('limit', limit.toString());
		if (eventType) params.set('event_type', eventType);
		const query = params.toString();
		return this.request(`/audit${query ? `?${query}` : ''}`);
	}

	async getAuditEntry(id: string): Promise<AuditEntry> {
		return this.request(`/audit/${id}`);
	}

	// System
	async getSystemStatus(): Promise<SystemStatus> {
		return this.request('/system/status');
	}

	async triggerPoll(): Promise<void> {
		return this.request('/system/poll', {
			method: 'POST'
		});
	}

	async enableScheduler(): Promise<void> {
		return this.request('/system/scheduler/enable', {
			method: 'POST'
		});
	}

	async disableScheduler(): Promise<void> {
		return this.request('/system/scheduler/disable', {
			method: 'POST'
		});
	}
}

export const api = new ApiClient();
