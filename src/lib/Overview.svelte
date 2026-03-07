<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade } from 'svelte/transition';
  import { currencySettings, appSettings, formatCurrency as formatCurrencyHelper } from './stores';
  import { 
    TrendingDown, 
    TrendingUp, 
    Receipt, 
    Calendar
  } from 'lucide-svelte';
  import Dropdown from './Dropdown.svelte';

  const dispatch = createEventDispatcher();

  export let monthlyBalance: number;
  export let allTimeBalance: number;
  export let selectedMonth: string;
  export let availableMonths: string[];
  export let transactions: Array<{
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
  }>;

  $: formatCurrency = (cents: number): string => {
    return formatCurrencyHelper(cents, $currencySettings);
  };

  function formatDateImpl(dateString: string, month: string, dateFormat: string): string {
    const date = new Date(dateString);
    const [selYear, selMo] = month.split('-').map(Number);
    const referenceDate = new Date(selYear, selMo, 0);
    const now = new Date();
    const anchor = now < referenceDate ? now : referenceDate;
    const diffDays = Math.floor((anchor.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    
    if (dateFormat === 'YYYY-MM-DD') {
      const y = date.getFullYear();
      const m = String(date.getMonth() + 1).padStart(2, '0');
      const d = String(date.getDate()).padStart(2, '0');
      return `${y}-${m}-${d}`;
    }
    const locale = dateFormat === 'DD/MM/YYYY' ? 'en-GB' : 'en-US';
    return new Intl.DateTimeFormat(locale, { month: 'short', day: 'numeric' }).format(date);
  }

  $: formatDate = (dateString: string): string => {
    return formatDateImpl(dateString, selectedMonth, $appSettings.dateFormat);
  };

  function formatTime(dateString: string): string {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    }).format(date);
  }

  function groupTransactions(txList: Array<{
    id: number;
    amount: number;
    description: string;
    category: string;
    date: string;
  }>, weekStart: 'sunday' | 'monday') {
    const [selYear, selMo] = selectedMonth.split('-').map(Number);
    const lastDayOfMonth = new Date(selYear, selMo, 0);
    const now = new Date();
    const anchor = now < lastDayOfMonth ? now : lastDayOfMonth;
    const today = new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate());
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const weekStartDate = new Date(today);
    const dayOfWeek = weekStartDate.getDay();
    const offset = weekStart === 'monday' ? (dayOfWeek === 0 ? 6 : dayOfWeek - 1) : dayOfWeek;
    weekStartDate.setDate(weekStartDate.getDate() - offset);

    const groups: { [key: string]: typeof txList } = {
      Today: [],
      Yesterday: [],
      'This Week': [],
      Older: []
    };

    txList.forEach(t => {
      const tDate = new Date(t.date);
      const tDay = new Date(tDate.getFullYear(), tDate.getMonth(), tDate.getDate());

      if (tDay.getTime() === today.getTime()) {
        groups.Today.push(t);
      } else if (tDay.getTime() === yesterday.getTime()) {
        groups.Yesterday.push(t);
      } else if (tDay >= weekStartDate) {
        groups['This Week'].push(t);
      } else {
        groups.Older.push(t);
      }
    });

    return Object.entries(groups).filter(([_, txs]) => txs.length > 0);
  }

  function getCategoryColor(category: string): string {
    const colors: Record<string, string> = {
      'Food & Dining': 'bg-orange-500',
      'Shopping': 'bg-purple-500',
      'Transportation': 'bg-blue-500',
      'Entertainment': 'bg-pink-500',
      'Bills & Utilities': 'bg-yellow-500',
      'Healthcare': 'bg-red-500',
      'Income': 'bg-green-500',
      'Other': 'bg-gray-500'
    };
    return colors[category] || colors['Other'];
  }

  $: totalSpent = transactions.reduce((sum, t) => t.amount < 0 ? sum + Math.abs(t.amount) : sum, 0);
  $: totalIncome = transactions.reduce((sum, t) => t.amount > 0 ? sum + t.amount : sum, 0);
  $: transactionCount = transactions.length;
  $: daysInMonth = (() => {
    const [y, m] = selectedMonth.split('-').map(Number);
    return new Date(y, m, 0).getDate();
  })();
  $: dailyAverage = transactionCount > 0 ? totalSpent / daysInMonth : 0;
  $: groupedTransactions = groupTransactions(transactions, $appSettings.weekStart);

  let pendingDeleteId: number | null = null;

  function handleDelete(id: number) {
    if ($appSettings.confirmBeforeDelete) {
      pendingDeleteId = id;
    } else {
      dispatch('delete', { id });
    }
  }

  function confirmDelete() {
    if (pendingDeleteId !== null) {
      dispatch('delete', { id: pendingDeleteId });
      pendingDeleteId = null;
    }
  }

  function formatMonthLabel(month: string): string {
    const [year, monthNum] = month.split('-');
    const date = new Date(parseInt(year), parseInt(monthNum) - 1);
    const now = new Date();
    const currentMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
    
    if (month === currentMonth) {
      return 'This Month';
    }
    
    return date.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
  }

  function handleMonthChange(event: CustomEvent) {
    dispatch('monthChange', { month: event.detail.value });
  }

  $: monthOptions = availableMonths.map(month => ({
    value: month,
    label: formatMonthLabel(month)
  }));
</script>

<div class="flex h-full w-full">
  <div class="flex-1 p-4 lg:p-8 space-y-4 lg:space-y-6 overflow-auto min-w-0">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-0">
      <div>
        <h2 class="text-2xl lg:text-3xl font-black text-white mb-1">Dashboard</h2>
        <p class="text-xs lg:text-sm text-gray-500">Track your spending</p>
      </div>
      <div class="flex items-center gap-2">
        <Calendar size={18} class="text-gray-400 hidden sm:block" />
        <div class="min-w-[160px] sm:min-w-[200px]">
          <Dropdown
            value={selectedMonth}
            options={monthOptions}
            on:change={handleMonthChange}
          />
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 lg:gap-4">
      <div class="bg-gray-950 rounded-xl p-6 border-2 {monthlyBalance >= 0 ? 'border-blue-500/30 hover:border-blue-500/50' : 'border-red-500/30 hover:border-red-500/50'} shadow-lg hover:scale-[1.02] transition-all duration-300 ease-out relative overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-br {monthlyBalance >= 0 ? 'from-blue-500/5' : 'from-red-500/5'} to-transparent pointer-events-none"></div>
        <div class="relative">
          <p class="text-gray-500 text-xs font-medium mb-2 lg:mb-3 uppercase tracking-wider">{formatMonthLabel(selectedMonth)}</p>
          <p class="text-3xl lg:text-4xl xl:text-5xl font-black tracking-tight mb-1 {monthlyBalance >= 0 ? 'text-white' : 'text-red-400'}" style="font-feature-settings: 'tnum';">
            {formatCurrency(monthlyBalance)}
          </p>
          <div class="flex items-center gap-2 mt-3">
            {#if monthlyBalance === 0}
              <span class="text-xs text-gray-500">Even steven</span>
            {:else if monthlyBalance > 0}
              <TrendingUp size={14} class="text-green-400" />
              <span class="text-xs text-gray-500">Looking good</span>
            {:else}
              <TrendingDown size={14} class="text-red-400" />
              <span class="text-xs text-gray-500">In the red</span>
            {/if}
          </div>
        </div>
      </div>

      <div class="bg-gray-950 rounded-xl p-6 border-2 {allTimeBalance >= 0 ? 'border-purple-500/30 hover:border-purple-500/50' : 'border-red-500/30 hover:border-red-500/50'} shadow-lg hover:scale-[1.02] transition-all duration-300 ease-out relative overflow-hidden">
        <div class="absolute inset-0 bg-gradient-to-br {allTimeBalance >= 0 ? 'from-purple-500/5' : 'from-red-500/5'} to-transparent pointer-events-none"></div>
        <div class="relative">
          <p class="text-gray-500 text-xs font-medium mb-2 lg:mb-3 uppercase tracking-wider">All Time</p>
          <p class="text-3xl lg:text-4xl xl:text-5xl font-black tracking-tight mb-1 {allTimeBalance >= 0 ? 'text-white' : 'text-red-400'}" style="font-feature-settings: 'tnum';">
            {formatCurrency(allTimeBalance)}
          </p>
          <div class="flex items-center gap-2 mt-3">
            {#if allTimeBalance === 0}
              <span class="text-xs text-gray-500">Balanced</span>
            {:else if allTimeBalance > 0}
              <TrendingUp size={14} class="text-green-400" />
              <span class="text-xs text-gray-500">Net positive</span>
            {:else}
              <TrendingDown size={14} class="text-red-400" />
              <span class="text-xs text-gray-500">Net negative</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-800">
        <h3 class="text-lg font-bold text-white">Recent Transactions</h3>
      </div>

      {#if transactions.length === 0}
        <div class="p-12 text-center">
          <div class="inline-flex p-4 bg-gray-800 rounded-full mb-4">
            <Receipt size={32} class="text-gray-600" />
          </div>
          <p class="text-gray-400 text-lg font-medium mb-2">No transactions yet</p>
          <p class="text-gray-600 text-sm">Press <kbd class="px-2 py-1 bg-gray-800 rounded text-xs">Ctrl+N</kbd> to add your first one</p>
        </div>
      {:else}
        <div>
          {#each groupedTransactions as [groupName, groupTxs]}
            <div class="border-b border-gray-800 last:border-b-0">
              <div class="px-6 py-3 bg-gray-800/50">
                <h4 class="text-xs font-bold text-gray-400 uppercase tracking-wider">{groupName}</h4>
              </div>
              <div class="divide-y divide-gray-800/50">
                {#each groupTxs as transaction, i}
                  <div class="px-6 py-4 hover:bg-gray-800/30 transition-all duration-200 group flex items-center gap-4">
                    <div class="flex-shrink-0">
                      <div class="w-2 h-2 rounded-full {getCategoryColor(transaction.category)}"></div>
                    </div>

                    <div class="flex-1 min-w-0">
                      <p class="text-white font-semibold truncate">{transaction.description || transaction.category || 'General Expense'}</p>
                      <div class="flex items-center gap-2 mt-0.5">
                        <span class="text-xs text-gray-500">{formatTime(transaction.date)}</span>
                        <span class="text-xs text-gray-700">•</span>
                        <span class="text-xs text-gray-500">{transaction.category}</span>
                      </div>
                    </div>

                    <div class="flex items-center gap-3">
                      <p class="text-lg font-mono {transaction.amount >= 0 ? 'text-green-400' : 'text-red-400'}" style="font-feature-settings: 'tnum';">
                        {transaction.amount >= 0 ? '+' : ''}{formatCurrency(transaction.amount)}
                      </p>
                      <button
                        on:click={() => dispatch('edit', { transaction })}
                        class="opacity-0 group-hover:opacity-100 p-2 hover:bg-blue-500/10 hover:scale-110 rounded-lg text-blue-400 transition-all duration-200"
                        title="Edit"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                      </button>
                      <button
                        on:click={() => handleDelete(transaction.id)}
                        class="opacity-0 group-hover:opacity-100 p-2 hover:bg-red-500/10 hover:scale-110 rounded-lg text-red-400 transition-all duration-200"
                        title="Delete"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="hidden xl:flex xl:w-80 2xl:w-96 bg-gray-900 border-l border-gray-800 flex-col h-full flex-shrink-0">
    <div class="p-4 xl:p-6 pb-4 flex-shrink-0">
      <h3 class="text-lg font-bold text-white">Statistics</h3>
    </div>

    <div class="flex-1 px-4 xl:px-6 pb-4 xl:pb-6 overflow-y-auto">
      <div class="space-y-3">
      <div class="bg-gray-800/50 rounded-xl p-5 border border-gray-700/50">
        <p class="text-gray-400 text-xs font-medium mb-4 uppercase tracking-wider">Cash Flow</p>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full bg-red-400"></div>
              <span class="text-sm text-gray-400">Spent</span>
            </div>
            <span class="text-lg font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(totalSpent)}</span>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full bg-green-400"></div>
              <span class="text-sm text-gray-400">Income</span>
            </div>
            <span class="text-lg font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(totalIncome)}</span>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">Daily Avg</p>
          <p class="text-xl font-mono text-white" style="font-feature-settings: 'tnum';">{formatCurrency(dailyAverage)}</p>
        </div>

        <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-700/50">
          <p class="text-gray-500 text-xs mb-2">Count</p>
          <p class="text-xl font-mono text-white" style="font-feature-settings: 'tnum';">{transactionCount}</p>
        </div>
      </div>
      </div>
    </div>
  </div>
</div>

{#if pendingDeleteId !== null}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[60] p-4" in:fade={{ duration: 150 }}>
    <div class="bg-gray-900 rounded-xl w-full max-w-sm border border-gray-700 shadow-2xl p-6 space-y-4">
      <h3 class="text-lg font-bold text-white">Delete Transaction</h3>
      <p class="text-sm text-gray-400">Are you sure? This action cannot be undone.</p>
      <div class="flex gap-3 pt-2">
        <button
          on:click={confirmDelete}
          class="flex-1 bg-red-600 hover:bg-red-700 text-white px-4 py-2.5 rounded-lg font-semibold transition-colors"
        >
          Delete
        </button>
        <button
          on:click={() => (pendingDeleteId = null)}
          class="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2.5 rounded-lg font-semibold transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
