<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { X, Settings as SettingsIcon, DollarSign, Globe, Check } from 'lucide-svelte';
  import { currencySettings, currencyOptions, type CurrencySettings } from './stores';
  import Dropdown from './Dropdown.svelte';

  const dispatch = createEventDispatcher();

  let selectedCurrency = $currencySettings.code;
  let activeTab: 'currency' | 'general' = 'currency';

  function handleCurrencyChange(event: CustomEvent) {
    const code = event.detail.value;
    const currency = currencyOptions.find(c => c.code === code);
    
    if (currency) {
      currencySettings.set({
        code: currency.code,
        symbol: currency.symbol,
        position: currency.position,
        locale: currency.locale
      });
      selectedCurrency = code;
    }
  }

  $: currencyDropdownOptions = currencyOptions.map(c => ({
    value: c.code,
    label: `${c.symbol} ${c.name} (${c.code})`
  }));

  $: selectedCurrencyOption = currencyOptions.find(c => c.code === selectedCurrency);
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
