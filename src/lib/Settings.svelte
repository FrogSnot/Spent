<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-shell';
  import { X, Settings as SettingsIcon, DollarSign, Globe, Check, Plus, Trash2, Database, Info, Keyboard, Copy, AlertTriangle, ExternalLink } from 'lucide-svelte';
  import { currencySettings, customCurrencies, allCurrencyOptions, appSettings, type CurrencySettings, type CurrencyOption, type AppSettings } from './stores';
  import Dropdown from './Dropdown.svelte';

  const dispatch = createEventDispatcher();

  let selectedCurrency = $currencySettings.code;
  let activeTab: 'currency' | 'general' | 'data' | 'about' = 'currency';

  let categories: string[] = [];
  let dbPath = '';
  let clearConfirmText = '';
  let isClearing = false;
  let copiedPath = false;

  const dateFormatOptions = [
    { value: 'MM/DD/YYYY', label: 'MM/DD/YYYY (03/07/2026)' },
    { value: 'DD/MM/YYYY', label: 'DD/MM/YYYY (07/03/2026)' },
    { value: 'YYYY-MM-DD', label: 'YYYY-MM-DD (2026-03-07)' },
  ];

  const limitOptions = [
    { value: 25, label: '25' },
    { value: 50, label: '50' },
    { value: 100, label: '100' },
    { value: 200, label: '200' },
    { value: 0, label: 'All' },
  ];

  async function loadCategories() {
    try {
      categories = await invoke<string[]>('get_categories');
    } catch (error) {
      console.error('Failed to load categories:', error);
    }
  }

  async function loadDbPath() {
    try {
      dbPath = await invoke<string>('get_db_path');
    } catch (error) {
      console.error('Failed to get DB path:', error);
    }
  }

  async function handleClearAllData() {
    if (clearConfirmText !== 'DELETE') return;
    isClearing = true;
    try {
      await invoke('clear_all_data');
      clearConfirmText = '';
      dispatch('dataCleared');
    } catch (error) {
      console.error('Failed to clear data:', error);
    } finally {
      isClearing = false;
    }
  }

  async function copyDbPath() {
    try {
      await navigator.clipboard.writeText(dbPath);
      copiedPath = true;
      setTimeout(() => copiedPath = false, 2000);
    } catch (error) {
      console.error('Failed to copy path:', error);
    }
  }

  function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    appSettings.update(s => ({ ...s, [key]: value }));
  }

  async function openGitHub() {
    try {
      await open('https://github.com/FrogSnot/Spent');
    } catch (error) {
      console.error('Failed to open GitHub:', error);
    }
  }

  onMount(() => {
    loadCategories();
    loadDbPath();
  });

  let showCustomForm = false;
  let newCurrency: Omit<CurrencyOption, 'custom'> & { custom: true } = {
    name: '',
    code: '',
    symbol: '',
    position: 'before',
    locale: 'en-US',
    custom: true,
  };
  let customError = '';

  function handleCurrencyChange(event: CustomEvent) {
    const code = event.detail.value;
    const currency = $allCurrencyOptions.find(c => c.code === code);
    if (currency) {
      currencySettings.set({
        code: currency.code,
        symbol: currency.symbol,
        position: currency.position,
        locale: currency.locale,
      });
      selectedCurrency = code;
    }
  }

  const positionOptions = [
    { value: 'before', label: 'Before amount' },
    { value: 'after', label: 'After amount' },
  ];

  function handlePositionChange(event: CustomEvent) {
    newCurrency.position = event.detail.value;
  }

  function addCustomCurrency() {
    const code = newCurrency.code.trim().toUpperCase();
    const name = newCurrency.name.trim();
    const symbol = newCurrency.symbol.trim();
    if (!code || !name || !symbol) {
      customError = 'Name, code, and symbol are required.';
      return;
    }
    if ($allCurrencyOptions.some(c => c.code === code)) {
      customError = `Currency code "${code}" already exists.`;
      return;
    }
    customCurrencies.update(list => [
      ...list,
      { name, code, symbol, position: newCurrency.position, locale: newCurrency.locale || 'en-US', custom: true },
    ]);
    newCurrency = { name: '', code: '', symbol: '', position: 'before', locale: 'en-US', custom: true };
    customError = '';
    showCustomForm = false;
  }

  function removeCustomCurrency(code: string) {
    customCurrencies.update(list => list.filter(c => c.code !== code));
    if (selectedCurrency === code) {
      selectedCurrency = 'USD';
      currencySettings.set({ code: 'USD', symbol: '$', position: 'before', locale: 'en-US' });
    }
  }

  $: currencyDropdownOptions = $allCurrencyOptions.map(c => ({
    value: c.code,
    label: `${c.symbol} ${c.name} (${c.code})`,
  }));

  $: selectedCurrencyOption = $allCurrencyOptions.find(c => c.code === selectedCurrency);
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" in:fade={{ duration: 200 }}>
  <div class="bg-gray-900 rounded-xl w-full max-w-3xl border border-gray-700 shadow-2xl overflow-hidden" in:scale={{ duration: 300, start: 0.95, easing: backOut }}>
    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-800 bg-gradient-to-r from-indigo-600 to-indigo-700">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-white/10 rounded-lg">
          <SettingsIcon size={20} class="text-white" />
        </div>
        <div>
          <h2 class="text-xl font-bold text-white">Settings</h2>
          <p class="text-sm text-indigo-100">Customize your experience</p>
        </div>
      </div>
      <button
        on:click={() => dispatch('close')}
        class="p-2 hover:bg-white/10 rounded-lg transition-colors"
      >
        <X size={20} class="text-white" />
      </button>
    </div>

    <div class="flex">
      <!-- Sidebar -->
      <div class="w-48 bg-gray-800/50 border-r border-gray-800 p-4">
        <nav class="space-y-1">
          <button
            on:click={() => activeTab = 'currency'}
            class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all {activeTab === 'currency'
              ? 'bg-indigo-600 text-white'
              : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
          >
            <DollarSign size={18} />
            <span class="text-sm font-medium">Currency</span>
          </button>
          
          <button
            on:click={() => activeTab = 'general'}
            class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all {activeTab === 'general'
              ? 'bg-indigo-600 text-white'
              : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
          >
            <Globe size={18} />
            <span class="text-sm font-medium">General</span>
          </button>
          
          <button
            on:click={() => activeTab = 'data'}
            class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all {activeTab === 'data'
              ? 'bg-indigo-600 text-white'
              : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
          >
            <Database size={18} />
            <span class="text-sm font-medium">Data</span>
          </button>
          
          <button
            on:click={() => activeTab = 'about'}
            class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all {activeTab === 'about'
              ? 'bg-indigo-600 text-white'
              : 'text-gray-400 hover:text-white hover:bg-gray-800'}"
          >
            <Info size={18} />
            <span class="text-sm font-medium">About</span>
          </button>
        </nav>
      </div>

      <!-- Content -->
      <div class="flex-1 p-6 max-h-[60vh] overflow-y-auto">
        {#if activeTab === 'currency'}
          <div class="space-y-6">
            <div>
              <h3 class="text-lg font-bold text-white mb-1">Currency Settings</h3>
              <p class="text-sm text-gray-400">Choose your preferred currency for displaying amounts</p>
            </div>

            <div class="space-y-4">
              <div>
                <!-- svelte-ignore a11y-label-has-associated-control -->
                <label class="block text-sm font-semibold text-gray-300 mb-2">
                  Select Currency
                </label>
                <Dropdown
                  value={selectedCurrency}
                  options={currencyDropdownOptions}
                  icon={DollarSign}
                  on:change={handleCurrencyChange}
                />
              </div>

              {#if selectedCurrencyOption}
                <div class="bg-gray-800 rounded-xl p-5 border border-gray-700">
                  <h4 class="text-sm font-semibold text-gray-300 mb-4">Preview</h4>
                  
                  <div class="space-y-3">
                    <div class="flex items-center justify-between p-3 bg-gray-900 rounded-lg">
                      <span class="text-sm text-gray-400">Positive amount:</span>
                      <span class="text-lg font-mono font-bold text-green-400">
                        {#if selectedCurrencyOption.position === 'before'}
                          {selectedCurrencyOption.symbol}1,234.56
                        {:else}
                          1,234.56 {selectedCurrencyOption.symbol}
                        {/if}
                      </span>
                    </div>
                    
                    <div class="flex items-center justify-between p-3 bg-gray-900 rounded-lg">
                      <span class="text-sm text-gray-400">Negative amount:</span>
                      <span class="text-lg font-mono font-bold text-red-400">
                        {#if selectedCurrencyOption.position === 'before'}
                          -{selectedCurrencyOption.symbol}567.89
                        {:else}
                          -567.89 {selectedCurrencyOption.symbol}
                        {/if}
                      </span>
                    </div>
                  </div>

                  <div class="mt-4 pt-4 border-t border-gray-700">
                    <div class="grid grid-cols-2 gap-3 text-xs">
                      <div>
                        <span class="text-gray-500">Currency Code:</span>
                        <p class="text-white font-medium">{selectedCurrencyOption.code}</p>
                      </div>
                      <div>
                        <span class="text-gray-500">Symbol:</span>
                        <p class="text-white font-medium">{selectedCurrencyOption.symbol}</p>
                      </div>
                      <div>
                        <span class="text-gray-500">Position:</span>
                        <p class="text-white font-medium capitalize">{selectedCurrencyOption.position}</p>
                      </div>
                      <div>
                        <span class="text-gray-500">Locale:</span>
                        <p class="text-white font-medium">{selectedCurrencyOption.locale}</p>
                      </div>
                    </div>
                  </div>
                </div>
              {/if}

              {#if $customCurrencies.length > 0}
                <div class="bg-gray-800 rounded-xl p-5 border border-gray-700">
                  <h4 class="text-sm font-semibold text-gray-300 mb-3">Custom Currencies</h4>
                  <div class="space-y-2">
                    {#each $customCurrencies as c}
                      <div class="flex items-center justify-between px-3 py-2 bg-gray-900 rounded-lg">
                        <span class="text-sm text-white">{c.symbol} {c.name} <span class="text-gray-500">({c.code})</span></span>
                        <button
                          on:click={() => removeCustomCurrency(c.code)}
                          class="p-1 hover:text-red-400 text-gray-500 transition-colors"
                          title="Remove"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              {#if showCustomForm}
                <div class="bg-gray-800 rounded-xl p-5 border border-gray-700 space-y-3">
                  <h4 class="text-sm font-semibold text-gray-300">Add Custom Currency</h4>
                  <div class="grid grid-cols-2 gap-3">
                    <div>
                      <label for="custom-name" class="block text-xs text-gray-400 mb-1">Name</label>
                      <input
                        id="custom-name"
                        bind:value={newCurrency.name}
                        placeholder="e.g. My Currency"
                        class="w-full bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                      />
                    </div>
                    <div>
                      <label for="custom-code" class="block text-xs text-gray-400 mb-1">Code</label>
                      <input
                        id="custom-code"
                        bind:value={newCurrency.code}
                        placeholder="e.g. XYZ"
                        maxlength="8"
                        class="w-full bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white uppercase focus:outline-none focus:border-indigo-500"
                      />
                    </div>
                    <div>
                      <label for="custom-symbol" class="block text-xs text-gray-400 mb-1">Symbol</label>
                      <input
                        id="custom-symbol"
                        bind:value={newCurrency.symbol}
                        placeholder="e.g. ¤"
                        class="w-full bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                      />
                    </div>
                    <div>
                      <!-- svelte-ignore a11y-label-has-associated-control -->
                      <label class="block text-xs text-gray-400 mb-1">Symbol position</label>
                      <Dropdown
                        value={newCurrency.position}
                        options={positionOptions}
                        on:change={handlePositionChange}
                      />
                    </div>
                    <div class="col-span-2">
                      <label for="custom-locale" class="block text-xs text-gray-400 mb-1">Locale <span class="text-gray-600">(optional, e.g. en-US)</span></label>
                      <input
                        id="custom-locale"
                        bind:value={newCurrency.locale}
                        placeholder="en-US"
                        class="w-full bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-indigo-500"
                      />
                    </div>
                  </div>
                  {#if customError}
                    <p class="text-xs text-red-400">{customError}</p>
                  {/if}
                  <div class="flex gap-2 pt-1">
                    <button
                      on:click={addCustomCurrency}
                      class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm rounded-lg font-semibold transition-all"
                    >
                      Save
                    </button>
                    <button
                      on:click={() => { showCustomForm = false; customError = ''; }}
                      class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white text-sm rounded-lg transition-all"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              {:else}
                <button
                  on:click={() => showCustomForm = true}
                  class="flex items-center gap-2 text-sm text-indigo-400 hover:text-indigo-300 transition-colors"
                >
                  <Plus size={16} />
                  Add custom currency
                </button>
              {/if}

              <div class="bg-blue-500/10 border border-blue-500/30 rounded-xl p-4">
                <div class="flex gap-3">
                  <div class="flex-shrink-0 mt-0.5">
                    <Check size={18} class="text-blue-400" />
                  </div>
                  <div class="text-sm text-blue-200">
                    <p class="font-medium mb-1">Changes apply immediately</p>
                    <p class="text-blue-300 text-xs">All amounts throughout the app will use your selected currency format. This is a display preference only and doesn't convert values.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

        {:else if activeTab === 'general'}
          <div class="space-y-6">
            <div>
              <h3 class="text-lg font-bold text-white mb-1">General Settings</h3>
              <p class="text-sm text-gray-400">Display preferences and behavior</p>
            </div>

            <div class="space-y-4">
              <h4 class="text-xs font-bold text-gray-500 uppercase tracking-wider">Display</h4>

              <div class="bg-gray-800 rounded-xl p-5 border border-gray-700 space-y-5">
                <div>
                  <!-- svelte-ignore a11y-label-has-associated-control -->
                  <label class="block text-sm font-semibold text-gray-300 mb-1">Date Format</label>
                  <p class="text-xs text-gray-500 mb-2">How dates appear throughout the app</p>
                  <Dropdown
                    value={$appSettings.dateFormat}
                    options={dateFormatOptions}
                    on:change={(e) => updateSetting('dateFormat', e.detail.value)}
                  />
                </div>

                <div>
                  <!-- svelte-ignore a11y-label-has-associated-control -->
                  <label class="block text-sm font-semibold text-gray-300 mb-1">Week Starts On</label>
                  <p class="text-xs text-gray-500 mb-2">First day of the week for grouping</p>
                  <div class="flex gap-2">
                    <button
                      on:click={() => updateSetting('weekStart', 'sunday')}
                      class="flex-1 px-4 py-2.5 rounded-lg font-medium text-sm transition-all {$appSettings.weekStart === 'sunday'
                        ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-600/20'
                        : 'bg-gray-900 text-gray-400 hover:bg-gray-700 border border-gray-700'}"
                    >
                      Sunday
                    </button>
                    <button
                      on:click={() => updateSetting('weekStart', 'monday')}
                      class="flex-1 px-4 py-2.5 rounded-lg font-medium text-sm transition-all {$appSettings.weekStart === 'monday'
                        ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-600/20'
                        : 'bg-gray-900 text-gray-400 hover:bg-gray-700 border border-gray-700'}"
                    >
                      Monday
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <div class="space-y-4">
              <h4 class="text-xs font-bold text-gray-500 uppercase tracking-wider">Behavior</h4>

              <div class="bg-gray-800 rounded-xl p-5 border border-gray-700 space-y-5">
                <div>
                  <!-- svelte-ignore a11y-label-has-associated-control -->
                  <label class="block text-sm font-semibold text-gray-300 mb-1">Default Category</label>
                  <p class="text-xs text-gray-500 mb-2">Pre-selected when adding new transactions</p>
                  <Dropdown
                    value={$appSettings.defaultCategory}
                    options={categories.map(c => ({ value: c, label: c }))}
                    on:change={(e) => updateSetting('defaultCategory', e.detail.value)}
                  />
                </div>

                <div>
                  <!-- svelte-ignore a11y-label-has-associated-control -->
                  <label class="block text-sm font-semibold text-gray-300 mb-1">Transaction Limit</label>
                  <p class="text-xs text-gray-500 mb-2">Max recent transactions shown on the dashboard</p>
                  <div class="flex gap-2">
                    {#each limitOptions as opt}
                      <button
                        on:click={() => updateSetting('transactionLimit', opt.value)}
                        class="flex-1 px-3 py-2.5 rounded-lg font-medium text-sm transition-all {$appSettings.transactionLimit === opt.value
                          ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-600/20'
                          : 'bg-gray-900 text-gray-400 hover:bg-gray-700 border border-gray-700'}"
                      >
                        {opt.label}
                      </button>
                    {/each}
                  </div>
                </div>

                <div class="flex items-center justify-between">
                  <div>
                    <!-- svelte-ignore a11y-label-has-associated-control -->
                    <label class="block text-sm font-semibold text-gray-300">Confirm Before Delete</label>
                    <p class="text-xs text-gray-500 mt-0.5">Ask for confirmation when deleting transactions</p>
                  </div>
                  <button
                    on:click={() => updateSetting('confirmBeforeDelete', !$appSettings.confirmBeforeDelete)}
                    class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {$appSettings.confirmBeforeDelete ? 'bg-indigo-600' : 'bg-gray-700'}"
                    role="switch"
                    aria-checked={$appSettings.confirmBeforeDelete}
                  >
                    <span
                      class="inline-block h-4 w-4 rounded-full bg-white transition-transform {$appSettings.confirmBeforeDelete ? 'translate-x-6' : 'translate-x-1'}"
                    />
                  </button>
                </div>
              </div>
            </div>
          </div>

        {:else if activeTab === 'data'}
          <div class="space-y-6">
            <div>
              <h3 class="text-lg font-bold text-white mb-1">Data Management</h3>
              <p class="text-sm text-gray-400">Database and storage information</p>
            </div>

            <div class="bg-gray-800 rounded-xl p-5 border border-gray-700 space-y-3">
              <h4 class="text-sm font-semibold text-gray-300">Database Location</h4>
              <div class="flex items-center gap-2">
                <div class="flex-1 bg-gray-900 rounded-lg px-4 py-2.5 text-sm text-gray-400 font-mono truncate border border-gray-700">
                  {dbPath || 'Loading...'}
                </div>
                <button
                  on:click={copyDbPath}
                  class="p-2.5 rounded-lg transition-all {copiedPath ? 'bg-green-600 text-white' : 'bg-gray-900 text-gray-400 hover:text-white hover:bg-gray-700 border border-gray-700'}"
                  title="Copy path"
                >
                  {#if copiedPath}
                    <Check size={16} />
                  {:else}
                    <Copy size={16} />
                  {/if}
                </button>
              </div>
              <p class="text-xs text-gray-600">All your data is stored locally in this SQLite file.</p>
            </div>

            <div class="bg-red-500/5 rounded-xl p-5 border border-red-500/20 space-y-4">
              <div class="flex items-center gap-3">
                <AlertTriangle size={20} class="text-red-400 flex-shrink-0" />
                <div>
                  <h4 class="text-sm font-semibold text-red-400">Danger Zone</h4>
                  <p class="text-xs text-gray-500 mt-0.5">This action cannot be undone</p>
                </div>
              </div>
              <p class="text-sm text-gray-400">Delete all transactions and custom containers. Default categories and the default container will be preserved.</p>
              <div class="space-y-3">
                <div>
                  <label for="clear-confirm" class="block text-xs text-gray-500 mb-1">Type <span class="text-red-400 font-mono font-bold">DELETE</span> to confirm</label>
                  <input
                    id="clear-confirm"
                    type="text"
                    bind:value={clearConfirmText}
                    placeholder="Type DELETE to confirm"
                    class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2.5 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-red-500 font-mono"
                    autocomplete="off"
                  />
                </div>
                <button
                  on:click={handleClearAllData}
                  disabled={clearConfirmText !== 'DELETE' || isClearing}
                  class="w-full px-4 py-2.5 rounded-lg font-semibold text-sm transition-all {clearConfirmText === 'DELETE' && !isClearing
                    ? 'bg-red-600 hover:bg-red-700 text-white'
                    : 'bg-gray-800 text-gray-600 cursor-not-allowed'}"
                >
                  {isClearing ? 'Clearing...' : 'Clear All Data'}
                </button>
              </div>
            </div>
          </div>

        {:else if activeTab === 'about'}
          <div class="space-y-6">
            <div>
              <h3 class="text-lg font-bold text-white mb-1">About Spent</h3>
              <p class="text-sm text-gray-400">Version and app information</p>
            </div>

            <div class="bg-gray-800 rounded-xl p-5 border border-gray-700">
              <div class="flex items-center gap-4">
                <div class="p-3 bg-indigo-600/20 rounded-xl">
                  <SettingsIcon size={28} class="text-indigo-400" />
                </div>
                <div>
                  <h4 class="text-lg font-bold text-white">Spent</h4>
                  <p class="text-sm text-gray-400">v1.1.9</p>
                  <p class="text-xs text-gray-600 mt-0.5">Minimalist, local-first finance tracker</p>
                </div>
              </div>
              <div class="mt-4 pt-4 border-t border-gray-700 text-center text-xs">
                <div>
                  <p class="text-gray-500">Made with love by FrogSnot ;)</p>
                </div>
              </div>
            </div>

            <div class="bg-gray-800 rounded-xl p-5 border border-gray-700">
              <div class="flex items-center gap-2 mb-4">
                <Keyboard size={16} class="text-gray-400" />
                <h4 class="text-sm font-semibold text-gray-300">Keyboard Shortcuts</h4>
              </div>
              <div class="space-y-2.5">
                {#each [
                  { keys: 'Ctrl + N', action: 'Quick add transaction' },
                  { keys: 'Ctrl + K', action: 'Open command palette' },
                  { keys: 'Ctrl + Enter', action: 'Submit form' },
                  { keys: 'Escape', action: 'Close dialogs' },
                ] as shortcut}
                  <div class="flex items-center justify-between">
                    <span class="text-sm text-gray-400">{shortcut.action}</span>
                    <kbd class="px-2.5 py-1 bg-gray-900 border border-gray-700 rounded-md text-xs text-gray-300 font-mono">{shortcut.keys}</kbd>
                  </div>
                {/each}
              </div>
            </div>

            <button
              on:click={openGitHub}
              class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-xl text-gray-300 text-sm font-medium transition-all"
            >
              <ExternalLink size={16} />
              View on GitHub
            </button>
          </div>
        {/if}
      </div>
    </div>

    <div class="px-6 py-4 border-t border-gray-800 bg-gray-900/50 flex justify-between items-center">
      <div class="text-xs text-gray-500">
        Settings are saved automatically
      </div>
      <button
        on:click={() => dispatch('close')}
        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-semibold transition-all"
      >
        Done
      </button>
    </div>
  </div>
</div>
