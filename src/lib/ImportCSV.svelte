<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { readTextFile } from '@tauri-apps/plugin-fs';
  import { X, Upload, FileText, CheckCircle, AlertCircle, Download } from 'lucide-svelte';
  import Dropdown from './Dropdown.svelte';

  const dispatch = createEventDispatcher();

  export let containerId: number;

  let csvContent = '';
  let fileName = '';
  let isProcessing = false;
  let step: 'upload' | 'preview' | 'mapping' | 'result' = 'upload';
  let previewRows: string[][] = [];
  let headers: string[] = [];
  let skipHeader = true;
  
  let amountColumn = 0;
  let descriptionColumn = 1;
  let categoryColumn = 2;
  let dateColumn = 3;

  let successCount = 0;
  let errorCount = 0;
  let errors: string[] = [];

  async function handleFileSelect() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'CSV Files',
          extensions: ['csv']
        }]
      });

      if (selected && typeof selected === 'string') {
        fileName = selected.split('/').pop() || selected.split('\\').pop() || 'file.csv';
        csvContent = await readTextFile(selected);
        parsePreview();
        step = 'mapping';
      }
    } catch (error) {
      console.error('Failed to read file:', error);
      step = 'upload';
    }
  }

  function parsePreview() {
    const lines = csvContent.split('\n').filter(line => line.trim().length > 0);
    previewRows = lines.slice(0, 6).map(line => {
      const result: string[] = [];
      let current = '';
      let inQuotes = false;
      
      for (let i = 0; i < line.length; i++) {
        const char = line[i];
        
        if (char === '"') {
          inQuotes = !inQuotes;
        } else if (char === ',' && !inQuotes) {
          result.push(current.trim());
          current = '';
        } else {
          current += char;
        }
      }
      result.push(current.trim());
      
      return result;
    });

    if (previewRows.length > 0) {
      headers = previewRows[0];
    }
  }

  async function handleImport() {
    if (!csvContent) return;
    
    isProcessing = true;
    step = 'result';

    try {
      const result = await invoke<{success_count: number, error_count: number, errors: string[]}>('import_csv', {
        csvContent,
        containerId,
        amountColumn,
        descriptionColumn,
        categoryColumn,
        dateColumn,
        skipHeader,
      });

      successCount = result.success_count;
      errorCount = result.error_count;
      errors = result.errors;

      if (successCount > 0) {
        dispatch('imported');
      }
    } catch (error) {
      console.error('Import failed:', error);
      errorCount = 1;
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  function handleClose() {
    if (successCount > 0) {
      dispatch('imported');
    }
    dispatch('close');
  }

  function resetImport() {
    csvContent = '';
    fileName = '';
    step = 'upload';
    previewRows = [];
    successCount = 0;
    errorCount = 0;
    errors = [];
  }

  $: columnOptions = headers.map((header, index) => ({
    value: index,
    label: `Column ${index + 1}: ${header.substring(0, 30)}${header.length > 30 ? '...' : ''}`
  }));

  $: maxColumns = headers.length > 0 ? headers.length : 10;
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" in:fade={{ duration: 200 }}>
  <div class="bg-gray-900 rounded-xl w-full max-w-4xl border border-gray-700 shadow-2xl overflow-hidden" in:scale={{ duration: 300, start: 0.95, easing: backOut }}>
    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-800 bg-gradient-to-r from-green-600 to-green-700">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-white/10 rounded-lg">
          <Upload size={20} class="text-white" />
        </div>
        <div>
          <h2 class="text-xl font-bold text-white">Import from CSV</h2>
          <p class="text-sm text-green-100">Import transactions from a CSV file</p>
        </div>
      </div>
      <button
        on:click={handleClose}
        class="p-2 hover:bg-white/10 rounded-lg transition-colors"
      >
        <X size={20} class="text-white" />
      </button>
    </div>

    <div class="p-6 max-h-[70vh] overflow-y-auto">
      {#if step === 'upload'}
        <div class="text-center py-12">
          <div class="inline-flex p-6 bg-green-500/10 rounded-2xl mb-6">
            <FileText size={48} class="text-green-400" />
          </div>
          <h3 class="text-xl font-bold text-white mb-2">Select a CSV File</h3>
          <p class="text-gray-400 mb-6">Choose a CSV file containing your transactions</p>
          <button
            on:click={handleFileSelect}
            class="px-6 py-3 bg-green-600 hover:bg-green-700 text-white rounded-xl font-semibold transition-all shadow-lg shadow-green-600/20"
          >
            <div class="flex items-center gap-2">
              <Upload size={18} />
              <span>Choose CSV File</span>
            </div>
          </button>
          
          <div class="mt-8 text-left bg-gray-800 rounded-xl p-4 max-w-md mx-auto">
            <h4 class="text-sm font-semibold text-gray-300 mb-2">CSV Format Requirements:</h4>
            <ul class="text-xs text-gray-400 space-y-1">
              <li>• Columns: Amount, Description, Category, Date</li>
              <li>• Amount: Can include currency symbols ($, €, £)</li>
              <li>• Date formats: YYYY-MM-DD, MM/DD/YYYY, DD/MM/YYYY</li>
              <li>• Negative amounts = expenses, positive = income</li>
            </ul>
          </div>
        </div>

      {:else if step === 'mapping'}
        <div class="space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-lg font-bold text-white">Map Your Columns</h3>
              <p class="text-sm text-gray-400">Match CSV columns to transaction fields</p>
            </div>
            <div class="text-sm text-gray-400">
              File: <span class="text-white font-medium">{fileName}</span>
            </div>
          </div>

          <div class="bg-gray-800 rounded-xl p-4 border border-gray-700">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={skipHeader}
                class="w-4 h-4 rounded bg-gray-700 border-gray-600 text-green-600 focus:ring-green-500"
              />
              <span class="text-sm text-gray-300">First row contains headers</span>
            </label>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <!-- svelte-ignore a11y-label-has-associated-control -->
              <label class="block text-sm font-semibold text-gray-300 mb-2">
                Amount Column *
              </label>
              <Dropdown
                bind:value={amountColumn}
                options={columnOptions}
                on:change={(e) => amountColumn = e.detail.value}
              />
            </div>

            <div>
              <!-- svelte-ignore a11y-label-has-associated-control -->
              <label class="block text-sm font-semibold text-gray-300 mb-2">
                Description Column *
              </label>
              <Dropdown
                bind:value={descriptionColumn}
                options={columnOptions}
                on:change={(e) => descriptionColumn = e.detail.value}
              />
            </div>

            <div>
              <!-- svelte-ignore a11y-label-has-associated-control -->
              <label class="block text-sm font-semibold text-gray-300 mb-2">
                Category Column *
              </label>
              <Dropdown
                bind:value={categoryColumn}
                options={columnOptions}
                on:change={(e) => categoryColumn = e.detail.value}
              />
            </div>

            <div>
              <!-- svelte-ignore a11y-label-has-associated-control -->
              <label class="block text-sm font-semibold text-gray-300 mb-2">
                Date Column *
              </label>
              <Dropdown
                bind:value={dateColumn}
                options={columnOptions}
                on:change={(e) => dateColumn = e.detail.value}
              />
            </div>
          </div>

          <div class="bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
            <div class="px-4 py-3 bg-gray-750 border-b border-gray-700">
              <h4 class="text-sm font-semibold text-gray-300">Preview (first 5 rows)</h4>
            </div>
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="bg-gray-750">
                  <tr>
                    {#each previewRows[0] || [] as header, i}
                      <th class="px-4 py-2 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">
                        {#if skipHeader}
                          {header}
                        {:else}
                          Col {i + 1}
                        {/if}
                      </th>
                    {/each}
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-700">
                  {#each previewRows.slice(skipHeader ? 1 : 0, 6) as row}
                    <tr class="hover:bg-gray-750">
                      {#each row as cell}
                        <td class="px-4 py-2 text-gray-300 whitespace-nowrap">
                          {cell.substring(0, 50)}{cell.length > 50 ? '...' : ''}
                        </td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <div class="flex gap-3">
            <button
              type="button"
              on:click={resetImport}
              class="px-6 py-3 bg-gray-800 hover:bg-gray-700 text-white rounded-xl font-semibold transition-all border border-gray-700"
            >
              Cancel
            </button>
            <button
              type="button"
              on:click={handleImport}
              class="flex-1 px-6 py-3 bg-green-600 hover:bg-green-700 text-white rounded-xl font-semibold transition-all shadow-lg shadow-green-600/20"
            >
              Import Transactions
            </button>
          </div>
        </div>

      {:else if step === 'result'}
        <div class="text-center py-12">
          {#if isProcessing}
            <div class="inline-flex p-6 bg-blue-500/10 rounded-2xl mb-6 animate-pulse">
              <Upload size={48} class="text-blue-400" />
            </div>
            <h3 class="text-xl font-bold text-white mb-2">Processing...</h3>
            <p class="text-gray-400">Importing your transactions</p>
          {:else}
            <div class="inline-flex p-6 bg-{errorCount === 0 ? 'green' : successCount > 0 ? 'yellow' : 'red'}-500/10 rounded-2xl mb-6">
              {#if errorCount === 0}
                <CheckCircle size={48} class="text-green-400" />
              {:else}
                <AlertCircle size={48} class="text-{successCount > 0 ? 'yellow' : 'red'}-400" />
              {/if}
            </div>
            
            <h3 class="text-2xl font-bold text-white mb-4">
              {#if errorCount === 0}
                Import Successful!
              {:else if successCount > 0}
                Partially Imported
              {:else}
                Import Failed
              {/if}
            </h3>

            <div class="grid grid-cols-2 gap-4 max-w-md mx-auto mb-6">
              <div class="bg-green-500/10 border border-green-500/30 rounded-xl p-4">
                <div class="text-3xl font-bold text-green-400">{successCount}</div>
                <div class="text-sm text-gray-400">Imported</div>
              </div>
              <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-4">
                <div class="text-3xl font-bold text-red-400">{errorCount}</div>
                <div class="text-sm text-gray-400">Failed</div>
              </div>
            </div>

            {#if errors.length > 0}
              <div class="bg-gray-800 rounded-xl border border-gray-700 p-4 max-w-2xl mx-auto text-left max-h-64 overflow-y-auto">
                <h4 class="text-sm font-semibold text-red-400 mb-3">Errors:</h4>
                <ul class="space-y-1 text-xs text-gray-400">
                  {#each errors.slice(0, 20) as error}
                    <li>• {error}</li>
                  {/each}
                  {#if errors.length > 20}
                    <li class="text-gray-500">... and {errors.length - 20} more</li>
                  {/if}
                </ul>
              </div>
            {/if}

            <div class="flex gap-3 justify-center mt-6">
              {#if successCount > 0 || errorCount > 0}
                <button
                  on:click={resetImport}
                  class="px-6 py-3 bg-gray-700 hover:bg-gray-600 text-white rounded-xl font-semibold transition-all"
                >
                  Import Another File
                </button>
              {/if}
              <button
                on:click={handleClose}
                class="px-6 py-3 bg-green-600 hover:bg-green-700 text-white rounded-xl font-semibold transition-all shadow-lg shadow-green-600/20"
              >
                Done
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>
