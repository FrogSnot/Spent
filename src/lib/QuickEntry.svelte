<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { X, DollarSign, Plus, Trash2 } from 'lucide-svelte';
  import Dropdown from './Dropdown.svelte';
  import { currencySettings } from './stores';

  const dispatch = createEventDispatcher();

  let amount = '';
  let description = '';
  let category = 'Other';
  let transactionType: 'expense' | 'income' = 'expense';
  let categories: string[] = [];
  let showAddCategory = false;
  let newCategoryName = '';

  async function loadCategories() {
    try {
      categories = await invoke<string[]>('get_categories');
    } catch (error) {
      console.error('Failed to load categories:', error);
      categories = ['Food & Dining', 'Transportation', 'Shopping', 'Entertainment', 'Bills & Utilities', 'Healthcare', 'Income', 'Other'];
    }
  }

  async function handleAddCategory() {
    if (!newCategoryName.trim()) return;
    
    try {
      await invoke('add_category', { name: newCategoryName.trim() });
      await loadCategories();
      category = newCategoryName.trim();
      newCategoryName = '';
      showAddCategory = false;
    } catch (error) {
      console.error('Failed to add category:', error);
      dispatch('error', { message: 'Failed to add category. It might already exist.' });
    }
  }

  let pendingDeleteCategory: string | null = null;

  async function handleDeleteCategory(categoryName: string) {
    pendingDeleteCategory = categoryName;
  }

  async function confirmDeleteCategory() {
    if (!pendingDeleteCategory) return;
    const categoryName = pendingDeleteCategory;
    pendingDeleteCategory = null;
    
    try {
      await invoke('delete_category', { name: categoryName });
      await loadCategories();
      if (category === categoryName) {
        category = 'Other';
      }
    } catch (error) {
      console.error('Failed to delete category:', error);
      dispatch('error', { message: 'Failed to delete category. Default categories cannot be deleted.' });
    }
  }

  function handleSubmit() {
    const parsedAmount = parseFloat(amount);
    if (isNaN(parsedAmount)) {
      return;
    }

    const finalAmount = transactionType === 'expense' ? -Math.abs(parsedAmount) : Math.abs(parsedAmount);

    dispatch('add', {
      amount: finalAmount,
      description: description.trim() || null,
      category: category || null,
    });

    amount = '';
    description = '';
    category = 'Other';
    transactionType = 'expense';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (showAddCategory) {
        showAddCategory = false;
      } else {
        dispatch('close');
      }
    }
    if (event.key === 'Enter' && event.ctrlKey && !showAddCategory) {
      handleSubmit();
    }
  }

  onMount(() => {
    loadCategories();
  });

  $: categoryOptions = categories.map(cat => ({ value: cat, label: cat }));

  function handleCategoryChange(event: CustomEvent) {
    category = event.detail.value;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" in:fade={{ duration: 200 }} out:fade={{ duration: 150 }}>
  <div class="bg-gray-900 rounded-2xl w-full max-w-md border border-gray-800 shadow-2xl overflow-visible">
    <div class="bg-gradient-to-r {transactionType === 'expense' ? 'from-red-600 to-red-700' : 'from-green-600 to-green-700'} px-6 py-5 flex items-center justify-between rounded-t-2xl">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-white/10 rounded-lg backdrop-blur-sm">
          <DollarSign class="text-white" size={20} />
        </div>
        <div>
          <h3 class="text-xl font-bold text-white">Quick Entry</h3>
          <p class="{transactionType === 'expense' ? 'text-red-100' : 'text-green-100'} text-xs">Add a transaction</p>
        </div>
      </div>
      <button
        on:click={() => dispatch('close')}
        class="p-2 hover:bg-white/10 rounded-lg transition-colors text-white"
      >
        <X size={20} />
      </button>
    </div>

    <form on:submit|preventDefault={handleSubmit} class="p-6 space-y-5 overflow-visible">
      <div class="overflow-visible">
        <label class="block text-sm font-semibold text-gray-300 mb-2">
          Type *
        </label>
        <div class="flex gap-2 p-1">
          <button
            type="button"
            on:click={() => (transactionType = 'expense')}
            class="flex-1 px-4 py-3 rounded-xl font-semibold transition-all duration-300 {transactionType === 'expense'
              ? 'bg-red-500 text-white shadow-lg shadow-red-500/20 scale-105'
              : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:scale-105'}"
          >
            Expense
          </button>
          <button
            type="button"
            on:click={() => (transactionType = 'income')}
            class="flex-1 px-4 py-3 rounded-xl font-semibold transition-all duration-300 {transactionType === 'income'
              ? 'bg-green-500 text-white shadow-lg shadow-green-500/20 scale-105'
              : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:scale-105'}"
          >
            Income
          </button>
        </div>
      </div>

      <div>
        <label for="amount" class="block text-sm font-semibold text-gray-300 mb-2">
          Amount *
        </label>
        <div class="relative">
          {#if $currencySettings.position === 'before'}
            <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
              <span class="text-gray-500 text-lg">{$currencySettings.symbol}</span>
            </div>
            <input
              id="amount"
              type="number"
              step="0.01"
              bind:value={amount}
              placeholder="0.00"
              class="w-full pl-10 pr-4 py-3 bg-gray-800 border border-gray-700 rounded-xl text-white text-lg font-semibold placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
              required
            />
          {:else}
            <input
              id="amount"
              type="number"
              step="0.01"
              bind:value={amount}
              placeholder="0.00"
              class="w-full pl-4 pr-10 py-3 bg-gray-800 border border-gray-700 rounded-xl text-white text-lg font-semibold placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
              required
            />
            <div class="absolute inset-y-0 right-0 pr-4 flex items-center pointer-events-none">
              <span class="text-gray-500 text-lg">{$currencySettings.symbol}</span>
            </div>
          {/if}
        </div>
      </div>

      <div>
        <label for="description" class="block text-sm font-semibold text-gray-300 mb-2">
          Description <span class="text-gray-600 font-normal">(optional)</span>
        </label>
        <input
          id="description"
          type="text"
          bind:value={description}
          placeholder="Coffee, groceries, salary..."
          class="w-full px-4 py-3 bg-gray-800 border-2 border-gray-700 rounded-xl text-white placeholder-gray-600 focus:outline-none focus:border-blue-500 transition-all"
        />
      </div>

      <div>
        <label for="category" class="block text-sm font-semibold text-gray-300 mb-2">
          Category
        </label>
        <Dropdown
          value={category}
          options={categoryOptions}
          on:change={handleCategoryChange}
        />
      </div>

      <div class="flex gap-3 pt-2">
        <button
          type="submit"
          class="flex-1 {transactionType === 'expense' ? 'bg-red-500 hover:bg-red-600 shadow-red-500/20' : 'bg-green-500 hover:bg-green-600 shadow-green-500/20'} text-white px-6 py-3 rounded-xl font-semibold transition-all shadow-lg"
        >
          Add {transactionType === 'expense' ? 'Expense' : 'Income'}
        </button>
        <button
          type="button"
          on:click={() => dispatch('close')}
          class="px-6 py-3 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-xl font-semibold transition-all border border-gray-700"
        >
          Cancel
        </button>
      </div>
      
      <p class="text-xs text-gray-600 text-center pt-2">
        Press <kbd class="px-2 py-1 bg-gray-800 rounded text-gray-400">Ctrl+Enter</kbd> to submit quickly
      </p>
    </form>
  </div>
</div>

{#if pendingDeleteCategory}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[60] p-4" in:fade={{ duration: 150 }}>
    <div class="bg-gray-900 rounded-xl w-full max-w-sm border border-gray-700 shadow-2xl p-6 space-y-4">
      <h3 class="text-lg font-bold text-white">Delete Category</h3>
      <p class="text-sm text-gray-400">Delete category "{pendingDeleteCategory}"?</p>
      <div class="flex gap-3 pt-2">
        <button
          on:click={confirmDeleteCategory}
          class="flex-1 bg-red-600 hover:bg-red-700 text-white px-4 py-2.5 rounded-lg font-semibold transition-colors"
        >
          Delete
        </button>
        <button
          on:click={() => (pendingDeleteCategory = null)}
          class="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2.5 rounded-lg font-semibold transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
