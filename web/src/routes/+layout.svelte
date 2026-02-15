<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import {
		LayoutDashboard,
		Wallet,
		Sparkles,
		History,
		FileText,
		Settings,
		Menu,
		X,
		Beaker
	} from 'lucide-svelte';

	interface NavItem {
		href: string;
		label: string;
		icon: typeof LayoutDashboard;
		demoOnly?: boolean;
	}

	const navItems: NavItem[] = [
		{ href: '/', label: 'Dashboard', icon: LayoutDashboard },
		{ href: '/accounts', label: 'Accounts', icon: Wallet },
		{ href: '/rules', label: 'Rules', icon: Sparkles },
		{ href: '/executions', label: 'Executions', icon: History },
		{ href: '/audit', label: 'Audit Log', icon: FileText },
		{ href: '/demo', label: 'Demo', icon: Beaker, demoOnly: true },
		{ href: '/settings', label: 'Settings', icon: Settings }
	];

	let sidebarOpen = $state(false);
	let demoMode = $state(false);

	let { children } = $props();

	onMount(async () => {
		// Enable dark mode by default
		document.documentElement.classList.add('dark');
		
		try {
			const status = await api.getServerStatus();
			demoMode = status.demo_mode;
		} catch {
			// Ignore errors - demo mode detection is best-effort
		}
	});

	function getVisibleNavItems() {
		return navItems.filter((item) => !item.demoOnly || demoMode);
	}
</script>

<svelte:head>
	<title>Autobank</title>
</svelte:head>

<div class="dark min-h-screen bg-gray-900">
	<!-- Demo mode banner -->
	{#if demoMode}
		<div class="bg-amber-500 text-amber-950 text-center py-1.5 text-sm font-medium">
			<Beaker class="h-4 w-4 inline-block mr-1 -mt-0.5" />
			Demo Mode - Using simulated bank data
		</div>
	{/if}

	<!-- Mobile header -->
	<header class="lg:hidden bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center justify-between">
		<h1 class="text-lg font-semibold text-gray-100">Autobank</h1>
		<button class="btn btn-ghost p-2" onclick={() => (sidebarOpen = true)}>
			<Menu class="h-5 w-5" />
		</button>
	</header>

	<!-- Mobile sidebar overlay -->
	{#if sidebarOpen}
		<div class="lg:hidden fixed inset-0 z-50">
			<div
				class="absolute inset-0 bg-black/75"
				onclick={() => (sidebarOpen = false)}
				role="button"
				tabindex="-1"
			></div>
			<aside class="absolute left-0 top-0 h-full w-64 bg-gray-800 shadow-xl">
				<div class="p-4 flex items-center justify-between border-b border-gray-700">
					<h1 class="text-lg font-semibold text-gray-100">Autobank</h1>
					<button class="btn btn-ghost p-1" onclick={() => (sidebarOpen = false)}>
						<X class="h-5 w-5" />
					</button>
				</div>
				<nav class="p-4 space-y-1">
					{#each getVisibleNavItems() as item}
						<a
							href={item.href}
							class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors
								{page.url.pathname === item.href
								? 'bg-primary-900/50 text-primary-400'
								: 'text-gray-400 hover:bg-gray-700 hover:text-gray-100'}"
							onclick={() => (sidebarOpen = false)}
						>
							<item.icon class="h-5 w-5" />
							{item.label}
						</a>
					{/each}
				</nav>
			</aside>
		</div>
	{/if}

	<!-- Desktop sidebar -->
	<aside class="hidden lg:fixed lg:inset-y-0 lg:flex lg:w-64 lg:flex-col">
		<div class="flex flex-col flex-grow bg-gray-800 border-r border-gray-700">
			<div class="p-6 border-b border-gray-700">
				<h1 class="text-xl font-bold text-gray-100">Autobank</h1>
				<p class="text-xs text-gray-500 mt-1">Rule-based automation</p>
			</div>
			<nav class="flex-1 p-4 space-y-1 overflow-y-auto">
				{#each getVisibleNavItems() as item}
					<a
						href={item.href}
						class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors
							{page.url.pathname === item.href
							? 'bg-primary-900/50 text-primary-400'
							: 'text-gray-400 hover:bg-gray-700 hover:text-gray-100'}"
					>
						<item.icon class="h-5 w-5" />
						{item.label}
					</a>
				{/each}
			</nav>
		</div>
	</aside>

	<!-- Main content -->
	<main class="lg:pl-64">
		<div class="p-4 sm:p-6 lg:p-8">
			{@render children()}
		</div>
	</main>
</div>
