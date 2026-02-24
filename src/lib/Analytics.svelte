<script lang="ts">
  import { PieChart, Package } from 'lucide-svelte';
  import { currencySettings, formatCurrency as formatCurrencyHelper } from './stores';

  export let categoryTotals: Array<[string, number]>;

  $: formatCurrency = (cents: number): string => {
    return formatCurrencyHelper(cents, $currencySettings);
  };

  function getCategoryColor(index: number): string {
    const colors = [
      'bg-blue-500',
      'bg-purple-500',
      'bg-pink-500',
      'bg-orange-500',
      'bg-green-500',
      'bg-yellow-500',
      'bg-red-500',
      'bg-indigo-500'
    ];
    return colors[index % colors.length];
  }

  function getCategoryTextColor(index: number): string {
    const colors = [
      'text-blue-400',
      'text-purple-400',
      'text-pink-400',
      'text-orange-400',
      'text-green-400',
      'text-yellow-400',
      'text-red-400',
      'text-indigo-400'
    ];
    return colors[index % colors.length];
  }

  $: totalSpent = categoryTotals.reduce((sum, [_, amount]) => sum + Math.abs(amount), 0);
  $: topCategory = categoryTotals.length > 0 ? categoryTotals[0] : null;
</script>

<div class="p-4 lg:p-8 space-y-4 lg:space-y-6 h-full overflow-auto">
  <div>
    <h2 class="text-2xl lg:text-3xl font-black text-white mb-1">Analytics</h2>
    <p class="text-xs lg:text-sm text-gray-500">Understand your spending patterns</p>
  </div>

  <div class="grid grid-cols-1 gap-4 lg:gap-6">
    <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg p-4 lg:p-6">
      <div class="flex items-center gap-3 mb-6">
        <div class="p-2 bg-purple-500/10 rounded-lg">
          <PieChart class="text-purple-400" size={20} />
        </div>
        <h3 class="text-lg font-bold text-white">Spending by Category</h3>
      </div>

      {#if categoryTotals.length === 0}
        <div class="py-12 text-center">
          <div class="inline-flex p-4 bg-gray-800 rounded-full mb-4">
            <Package size={32} class="text-gray-600" />
          </div>
          <p class="text-gray-500">No spending data yet</p>
          <p class="text-gray-600 text-sm mt-1">Add some transactions to see insights</p>
        </div>
      {:else}
        <div class="space-y-4">
          {#each categoryTotals.slice(0, 6) as [category, amount], i}
            {@const percentage = (Math.abs(amount) / totalSpent) * 100}
            {@const isTopCategory = i === 0}
            <div class="{isTopCategory ? 'bg-gray-800/50 p-4 rounded-lg border border-gray-700' : ''}">
              <div class="flex items-center justify-between mb-2">
                <div class="flex items-center gap-2">
                  <div class="w-2 h-2 rounded-full {getCategoryColor(i)}"></div>
                  <span class="text-sm font-medium text-gray-300">{category}</span>
                  {#if isTopCategory}
                    <span class="text-xs px-2 py-0.5 bg-red-500/20 text-red-400 rounded-full">Top Spending</span>
                  {/if}
                </div>
                <div class="text-right">
                  <p class="text-sm font-mono font-bold text-white" style="font-feature-settings: 'tnum';">{formatCurrency(amount)}</p>
                  <p class="text-xs text-gray-500">{percentage.toFixed(1)}%</p>
                </div>
              </div>
              <div class="h-1.5 bg-gray-800 rounded-full overflow-hidden">
                <div
                  class="{getCategoryColor(i)} h-full rounded-full transition-all duration-500"
                  style="width: {percentage}%"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="bg-gray-900 rounded-xl border border-gray-800 shadow-lg p-6">
      <div class="flex items-center gap-3 mb-6">
        <div class="p-2 bg-blue-500/10 rounded-lg">
          <PieChart class="text-blue-400" size={20} />
        </div>
        <h3 class="text-lg font-bold text-white">Category Distribution</h3>
      </div>

      {#if categoryTotals.length > 0}
        <div class="flex flex-col lg:flex-row items-center justify-center gap-6 lg:gap-12">
          <svg width="240" height="240" viewBox="0 0 240 240" class="transform -rotate-90 flex-shrink-0">
            <circle
              cx="120"
              cy="120"
              r="100"
              fill="none"
              stroke="rgba(55, 65, 81, 0.3)"
              stroke-width="40"
            />
            {#each categoryTotals.slice(0, 8) as [_, amount], i}
              {@const offset = categoryTotals.slice(0, i).reduce((sum, [_, amt]) => sum + (Math.abs(amt) / totalSpent) * 628.3, 0)}
              {@const dashLength = (Math.abs(amount) / totalSpent) * 628.3}
              {@const colorMap = [
                '#3b82f6', '#a855f7', '#ec4899', '#f97316',
                '#22c55e', '#eab308', '#ef4444', '#6366f1'
              ]}
              <circle
                cx="120"
                cy="120"
                r="100"
                fill="none"
                stroke={colorMap[i % colorMap.length]}
                stroke-width="40"
                stroke-dasharray="{dashLength} 628.3"
                stroke-dashoffset={-offset}
                opacity="0.8"
              />
            {/each}
          </svg>

          <div class="space-y-3">
            {#each categoryTotals.slice(0, 8) as [category, amount], i}
              {@const percentage = (Math.abs(amount) / totalSpent) * 100}
              <div class="flex items-center gap-3">
                <div class="w-4 h-4 rounded {getCategoryColor(i)}"></div>
                <div class="flex-1">
                  <p class="text-sm font-medium text-gray-300">{category}</p>
                  <p class="text-xs text-gray-500">{percentage.toFixed(1)}%</p>
                </div>
                <p class="text-sm font-bold {getCategoryTextColor(i)}">{formatCurrency(amount)}</p>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="py-12 text-center">
          <p class="text-gray-500">No spending data to visualize</p>
        </div>
      {/if}
    </div>
  </div>
</div>
