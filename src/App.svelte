<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { open } from '@tauri-apps/plugin-shell';
  import { LayoutDashboard, TrendingUp, Plus, Settings as SettingsIcon, Github, Wallet } from 'lucide-svelte';
  import Overview from './lib/Overview.svelte';
  import Analytics from './lib/Analytics.svelte';
  import QuickEntry from './lib/QuickEntry.svelte';
  import EditTransaction from './lib/EditTransaction.svelte';
  import CommandPalette from './lib/CommandPalette.svelte';
  import CategoryManager from './lib/CategoryManager.svelte';
  import ContainerManager from './lib/ContainerManager.svelte';
  import ImportCSV from './lib/ImportCSV.svelte';
  import Settings from './lib/Settings.svelte';
  import Dropdown from './lib/Dropdown.svelte';
  import Toast from './lib/Toast.svelte';

  interface Transaction {
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
    container_id: number;
  }

  interface Container {
    id: number;
    name: string;
    created_at: string;
    is_default: boolean;
  }

  let activeTab: 'overview' | 'analytics' = 'overview';
  let showQuickEntry = false;
  let showEditTransaction = false;
  let editingTransaction: Transaction | null = null;
  let showCommandPalette = false;
  let showCategoryManager = false;
  let showContainerManager = false;
  let showImportCSV = false;
  let showSettings = false;
  let monthlyBalance = 0;
  let allTimeBalance = 0;
  let transactions: Transaction[] = [];
  let categoryTotals: Array<[string, number]> = [];
  let availableMonths: string[] = [];
  let selectedMonth: string = '';
  let containers: Container[] = [];
  let selectedContainer: Container | null = null;
  let toastMessage = '';
  let toastType: 'success' | 'error' | 'info' | 'warning' = 'info';
  let showToast = false;

  function getCurrentMonth() {
    const now = new Date();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
  }

  async function loadContainers() {
    try {
      containers = await invoke<Container[]>('get_containers');
      if (containers.length > 0 && !selectedContainer) {
        selectedContainer = containers.find(c => c.is_default) || containers[0];
      }
    } catch (error) {
      console.error('Failed to load containers:', error);
    }
  }

  async function loadAvailableMonths() {
    if (!selectedContainer) return;
    
    try {
      const months = await invoke<string[]>('get_available_months', { containerId: selectedContainer.id });
      const currentMonth = getCurrentMonth();
      
      const monthSet = new Set(months);
      monthSet.add(currentMonth);
      
      availableMonths = Array.from(monthSet).sort((a, b) => b.localeCompare(a));
      
      if (!selectedMonth) {
        selectedMonth = currentMonth;
      }
    } catch (error) {
      console.error('Failed to load months:', error);
      availableMonths = [getCurrentMonth()];
      selectedMonth = getCurrentMonth();
    }
  }

  async function loadData() {
    if (!selectedContainer) return;
    
    try {
      if (selectedMonth === getCurrentMonth()) {
        monthlyBalance = await invoke<number>('get_monthly_balance', { containerId: selectedContainer.id });
        transactions = await invoke<Transaction[]>('get_transactions', { containerId: selectedContainer.id, limit: 50 });
        categoryTotals = await invoke<Array<[string, number]>>('get_category_totals', { containerId: selectedContainer.id });
      } else {
        monthlyBalance = await invoke<number>('get_balance_for_month', { containerId: selectedContainer.id, month: selectedMonth });
        transactions = await invoke<Transaction[]>('get_transactions_for_month', { containerId: selectedContainer.id, month: selectedMonth, limit: 50 });
        categoryTotals = await invoke<Array<[string, number]>>('get_category_totals_for_month', { containerId: selectedContainer.id, month: selectedMonth });
      }
      allTimeBalance = await invoke<number>('get_all_time_balance', { containerId: selectedContainer.id });
    } catch (error) {
      console.error('Failed to load data:', error);
    }
  }

  let _lastMonth = '';
  let _lastContainerId: number | null = null;

  $: if (selectedMonth && selectedMonth !== _lastMonth) {
    _lastMonth = selectedMonth;
    loadData();
  }

  $: if (selectedContainer && selectedContainer.id !== _lastContainerId) {
    _lastContainerId = selectedContainer.id;
    _lastMonth = '';
    loadAvailableMonths().then(() => loadData());
  }

  async function handleAddTransaction(event: CustomEvent) {
    if (!selectedContainer) return;
    
    const { amount, description, category } = event.detail;
    try {
      await invoke('add_transaction', {
        amount: Math.round(amount * 100),
        description: description || null,
        category: category || null,
        containerId: selectedContainer.id,
      });
      await loadData();
      showQuickEntry = false;
    } catch (error) {
      console.error('Failed to add transaction:', error);
      toastMessage = 'Failed to add transaction. Please try again.';
      toastType = 'error';
      showToast = true;
    }
  }

  async function handleDeleteTransaction(event: CustomEvent) {
    try {
      await invoke('delete_transaction', { id: event.detail.id });
      await loadData();
    } catch (error) {
      console.error('Failed to delete transaction:', error);
      toastMessage = 'Failed to delete transaction. Please try again.';
      toastType = 'error';
      showToast = true;
    }
  }

  function handleEditTransaction(event: CustomEvent) {
    editingTransaction = event.detail.transaction;
    showEditTransaction = true;
  }

  async function handleUpdateTransaction(event: CustomEvent) {
    const { id, amount, description, category } = event.detail;
    try {
      await invoke('update_transaction', {
        id,
        amount,
        description,
        category,
      });
      await loadData();
      showEditTransaction = false;
      editingTransaction = null;
    } catch (error) {
      console.error('Failed to update transaction:', error);
      toastMessage = 'Failed to update transaction. Please try again.';
      toastType = 'error';
      showToast = true;
    }
  }

  async function handleExport() {
    if (!selectedContainer) return;
    
    try {
      const csv = await invoke<string>('export_csv', { containerId: selectedContainer.id });
      const path = await save({
        defaultPath: `spent-${selectedContainer.name}-export.csv`,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });
      
      if (path) {
        await writeTextFile(path, csv);
        toastMessage = `CSV exported successfully to:\n${path}`;
        toastType = 'success';
        showToast = true;
      }
    } catch (error) {
      console.error('Export failed:', error);
      toastMessage = 'Failed to export CSV. Please try again.';
      toastType = 'error';
      showToast = true;
    }
  }

  async function openGitHub() {
    try {
      await open('https://github.com/FrogSnot/Spent');
    } catch (error) {
      console.error('Failed to open GitHub:', error);
    }
  }

  function handleCommand(event: CustomEvent) {
    const { action } = event.detail;
    showCommandPalette = false;
    
    switch (action) {
      case 'add':
        showQuickEntry = true;
        break;
      case 'categories':
        showCategoryManager = true;
        break;
      case 'containers':
        showContainerManager = true;
        break;
      case 'import':
        showImportCSV = true;
        break;
      case 'settings':
        showSettings = true;
        break;
      case 'export':
        handleExport();
        break;
      case 'analytics':
        activeTab = 'analytics';
        break;
      case 'transactions':
        activeTab = 'overview';
        break;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
      event.preventDefault();
      showCommandPalette = !showCommandPalette;
    }
    if ((event.ctrlKey || event.metaKey) && event.key === 'n') {
      event.preventDefault();
      showQuickEntry = !showQuickEntry;
    }
    if (event.key === 'Escape') {
      showQuickEntry = false;
      showCommandPalette = false;
      showCategoryManager = false;
      showContainerManager = false;
      showImportCSV = false;
      showSettings = false;
    }
  }

  function handleContainerChange(event: CustomEvent) {
    const containerId = event.detail.value as number;
    selectedContainer = containers.find(c => c.id === containerId) || null;
  }

  $: containerOptions = containers.map(c => ({
    value: c.id,
    label: c.is_default ? `${c.name} (Default)` : c.name
  }));

  function handleContainerDeleted(event: CustomEvent) {
    const deletedId = event.detail.id;
    if (selectedContainer?.id === deletedId) {
      selectedContainer = containers.find(c => c.is_default) || containers[0] || null;
    }
  }

  onMount(() => {
    loadContainers();
    loadAvailableMonths();
    loadData();
    
    const handleKeydownEvent = (event: KeyboardEvent) => handleKeydown(event);
    window.addEventListener('keydown', handleKeydownEvent);
    
    return () => window.removeEventListener('keydown', handleKeydownEvent);
  });
</script>

<main class="min-h-screen bg-gray-950 text-gray-100 flex">
  <aside class="w-56 lg:w-64 bg-gray-900 border-r border-gray-800 flex flex-col flex-shrink-0 hidden sm:flex">
    <div class="p-6 border-b border-gray-800">
      <h1 class="text-2xl font-black text-white tracking-tight">Spent</h1>
      <p class="text-xs text-gray-500 mt-1">Finance Tracker</p>
    </div>

    {#if selectedContainer}
      <div class="px-4 pt-4 pb-2">
        <!-- svelte-ignore a11y-label-has-associated-control -->
        <label class="block text-xs font-semibold text-gray-500 mb-2 uppercase tracking-wider">Active Container</label>
        <Dropdown
          value={selectedContainer.id}
          options={containerOptions}
          icon={Wallet}
          on:change={handleContainerChange}
        />
      </div>
    {/if}

    <nav class="flex-1 p-4 space-y-1">
      <button
        on:click={() => (activeTab = 'overview')}
        class="w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all {activeTab === 'overview'
          ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20'
          : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
      >
        <LayoutDashboard size={20} />
        <span class="font-medium">Overview</span>
      </button>

      <button
        on:click={() => (activeTab = 'analytics')}
        class="w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all {activeTab === 'analytics'
          ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20'
          : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
      >
        <TrendingUp size={20} />
        <span class="font-medium">Analytics</span>
      </button>
    </nav>

    <div class="p-4 space-y-3 border-t border-gray-800">
      <button
        on:click={() => (showQuickEntry = true)}
        class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-all shadow-lg shadow-blue-600/20"
      >
        <Plus size={20} />
        Quick Add
      </button>
      
      <button
        on:click={() => (showCommandPalette = true)}
        class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg font-medium transition-all text-sm"
      >
        <SettingsIcon size={16} />
        Commands
      </button>
      
      <div class="flex items-center justify-between px-2">
        <button
          on:click={openGitHub}
          class="flex items-center gap-1.5 text-gray-500 hover:text-gray-300 transition-colors text-xs hover:underline cursor-pointer"
        >
          <Github size={14} />
          <span>v1.1.9</span>
        </button>
        <button
          on:click={handleExport}
          class="text-xs text-gray-500 hover:text-gray-300 transition-colors"
        >
          Export CSV
        </button>
      </div>
    </div>
  </aside>

  <div class="flex-1 overflow-hidden flex flex-col">
    {#if activeTab === 'overview'}
      <Overview
        {monthlyBalance}
        {allTimeBalance}
        {transactions}
        {selectedMonth}
        {availableMonths}
        on:delete={handleDeleteTransaction}
        on:edit={handleEditTransaction}
        on:refresh={loadData}
        on:monthChange={(e) => selectedMonth = e.detail.month}
      />
    {:else if activeTab === 'analytics'}
      <Analytics {categoryTotals} />
    {/if}
  </div>

  {#if showQuickEntry}
    <QuickEntry
      on:add={handleAddTransaction}
      on:close={() => (showQuickEntry = false)}
    />
  {/if}

  {#if showCommandPalette}
    <CommandPalette
      on:command={handleCommand}
      on:close={() => (showCommandPalette = false)}
    />
  {/if}

  {#if showCategoryManager}
    <CategoryManager
      on:close={() => (showCategoryManager = false)}
    />
  {/if}

  {#if showContainerManager}
    <ContainerManager
      on:close={() => {
        showContainerManager = false;
        loadContainers();
      }}
      on:containerDeleted={handleContainerDeleted}
      on:containerUpdated={loadContainers}
    />
  {/if}

  {#if showEditTransaction && editingTransaction}
    <EditTransaction
      transaction={editingTransaction}
      on:save={handleUpdateTransaction}
      on:close={() => {
        showEditTransaction = false;
        editingTransaction = null;
      }}
    />
  {/if}

  {#if showImportCSV && selectedContainer}
    <ImportCSV
      containerId={selectedContainer.id}
      on:imported={loadData}
      on:close={() => (showImportCSV = false)}
    />
  {/if}

  {#if showSettings}
    <Settings
      on:close={() => (showSettings = false)}
    />
  {/if}

  {#if showToast}
    <Toast
      message={toastMessage}
      type={toastType}
      onClose={() => (showToast = false)}
    />
  {/if}
</main>
