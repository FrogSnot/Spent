<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { X, Settings as SettingsIcon, DollarSign, Globe, Check, Plus, Trash2 } from 'lucide-svelte';
  import { currencySettings, customCurrencies, allCurrencyOptions, type CurrencySettings, type CurrencyOption } from './stores';
  import Dropdown from './Dropdown.svelte';

  const dispatch = createEventDispatcher();

  let selectedCurrency = $currencySettings.code;
  let activeTab: 'currency' | 'general' = 'currency';

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
              <p class="text-sm text-gray-400">App preferences and configurations</p>
            </div>

            <div class="bg-gray-800 rounded-xl p-5 border border-gray-700">
              <div class="text-center py-8">
                <div class="inline-flex p-4 bg-gray-700 rounded-full mb-4">
                  <Globe size={32} class="text-gray-500" />
                </div>
                <p class="text-gray-400">More settings coming soon...</p>
                <p class="text-gray-600 text-sm mt-2">Future options: themes, date formats, backup settings, etc.</p>
              </div>
            </div>
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
