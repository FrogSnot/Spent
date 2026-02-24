<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { Search, Plus, Download, TrendingUp, Receipt, Tag, Wallet, Upload, Settings } from 'lucide-svelte';

  const dispatch = createEventDispatcher();

  let searchQuery = '';
  let selectedIndex = 0;

  const commands = [
    { id: 'add', label: 'Add Transaction', icon: Plus, action: 'add' },
    { id: 'import', label: 'Import from CSV', icon: Upload, action: 'import' },
    { id: 'categories', label: 'Manage Categories', icon: Tag, action: 'categories' },
    { id: 'containers', label: 'Manage Containers', icon: Wallet, action: 'containers' },
    { id: 'settings', label: 'Settings', icon: Settings, action: 'settings' },
    { id: 'export', label: 'Export to CSV', icon: Download, action: 'export' },
    { id: 'analytics', label: 'View Analytics', icon: TrendingUp, action: 'analytics' },
    { id: 'transactions', label: 'View Transactions', icon: Receipt, action: 'transactions' },
  ];

  $: filteredCommands = commands.filter(cmd =>
    cmd.label.toLowerCase().includes(searchQuery.toLowerCase())
  );

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selectedIndex = (selectedIndex + 1) % filteredCommands.length;
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      selectedIndex = selectedIndex === 0 ? filteredCommands.length - 1 : selectedIndex - 1;
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (filteredCommands[selectedIndex]) {
        dispatch('command', { action: filteredCommands[selectedIndex].action });
      }
    } else if (event.key === 'Escape') {
      dispatch('close');
    }
  }

  function selectCommand(index: number) {
    dispatch('command', { action: filteredCommands[index].action });
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-start justify-center z-50 p-4 pt-[15vh]" in:fade={{ duration: 150 }}>
  <div class="bg-gray-900 rounded-xl w-full max-w-2xl border border-gray-700 shadow-2xl overflow-hidden" in:fly={{ y: -20, duration: 300, easing: cubicOut }}>
    <div class="flex items-center gap-3 px-4 py-4 border-b border-gray-800">
      <Search size={20} class="text-gray-500" />
      <!-- svelte-ignore a11y-autofocus -->
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Type a command or search..."
        class="flex-1 bg-transparent text-white placeholder-gray-500 outline-none text-lg"
        autofocus
      />
      <kbd class="px-2 py-1 text-xs bg-gray-800 text-gray-400 rounded">Esc</kbd>
    </div>

    <div class="max-h-96 overflow-y-auto overflow-x-hidden">
      {#if filteredCommands.length === 0}
        <div class="px-4 py-8 text-center text-gray-500">
          No commands found
        </div>
      {:else}
        {#each filteredCommands as command, i}
          <button
            on:click={() => selectCommand(i)}
            class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-800 hover:scale-[1.01] transition-all duration-200 {i === selectedIndex ? 'bg-gray-800' : ''}"
          >
            <div class="p-2 bg-gray-800 rounded-lg">
              <svelte:component this={command.icon} size={18} class="text-gray-400" />
            </div>
            <span class="text-white font-medium flex-1 text-left">{command.label}</span>
            {#if i === selectedIndex}
              <kbd class="px-2 py-1 text-xs bg-gray-900 text-gray-400 rounded border border-gray-700">Enter</kbd>
            {/if}
          </button>
        {/each}
      {/if}
    </div>

    <div class="px-4 py-3 border-t border-gray-800 bg-gray-900/50">
      <div class="flex items-center gap-4 text-xs text-gray-500">
        <span class="flex items-center gap-1.5">
          <kbd class="px-2 py-1 bg-gray-800 rounded">↑↓</kbd> Navigate
        </span>
        <span class="flex items-center gap-1.5">
          <kbd class="px-2 py-1 bg-gray-800 rounded">Enter</kbd> Select
        </span>
        <span class="flex items-center gap-1.5">
          <kbd class="px-2 py-1 bg-gray-800 rounded">Esc</kbd> Close
        </span>
      </div>
    </div>
  </div>
</div>
