<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { backOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import { X, Plus, Trash2, Wallet, Edit2, Check } from 'lucide-svelte';

  const dispatch = createEventDispatcher();

  interface Container {
    id: number;
    name: string;
    created_at: string;
    is_default: boolean;
  }

  let containers: Container[] = [];
  let newContainerName = '';
  let isAdding = false;
  let editingId: number | null = null;
  let editingName = '';
  let errorMessage = '';
  let pendingDeleteContainer: Container | null = null;

  async function loadContainers() {
    try {
      containers = await invoke<Container[]>('get_containers');
    } catch (error) {
      console.error('Failed to load containers:', error);
    }
  }

  async function handleAddContainer() {
    if (!newContainerName.trim()) return;
    
    try {
      await invoke('add_container', { name: newContainerName.trim() });
      await loadContainers();
      newContainerName = '';
      isAdding = false;
      errorMessage = '';
    } catch (error) {
      console.error('Failed to add container:', error);
      errorMessage = 'Failed to add container. It might already exist.';
    }
  }

  async function handleDeleteContainer(container: Container) {
    if (container.is_default) {
      errorMessage = 'Cannot delete the default container.';
      return;
    }
    pendingDeleteContainer = container;
  }

  async function confirmDeleteContainer() {
    if (!pendingDeleteContainer) return;
    const container = pendingDeleteContainer;
    pendingDeleteContainer = null;
    
    try {
      await invoke('delete_container', { id: container.id });
      await loadContainers();
      dispatch('containerDeleted', { id: container.id });
    } catch (error) {
      console.error('Failed to delete container:', error);
      errorMessage = 'Failed to delete container.';
    }
  }

  function startEditing(container: Container) {
    editingId = container.id;
    editingName = container.name;
  }

  async function saveEdit(id: number) {
    if (!editingName.trim()) {
      editingId = null;
      return;
    }

    try {
      await invoke('update_container', { id, name: editingName.trim() });
      await loadContainers();
      editingId = null;
      dispatch('containerUpdated');
    } catch (error) {
      console.error('Failed to update container:', error);
      errorMessage = 'Failed to update container name.';
      editingId = null;
    }
  }

  function cancelEdit() {
    editingId = null;
    editingName = '';
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric', 
      year: 'numeric' 
    });
  }

  onMount(() => {
    loadContainers();
  });
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4" in:fade={{ duration: 200 }}>
  <div class="bg-gray-900 rounded-xl w-full max-w-2xl border border-gray-700 shadow-2xl overflow-hidden" in:scale={{ duration: 300, start: 0.95, easing: backOut }}>
    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-800">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-purple-600 rounded-lg">
          <Wallet size={20} class="text-white" />
        </div>
        <div>
          <h2 class="text-xl font-bold text-white">Manage Containers</h2>
          <p class="text-sm text-gray-500">Create and organize separate balance containers</p>
        </div>
      </div>
      <button
        on:click={() => dispatch('close')}
        class="p-2 hover:bg-gray-800 rounded-lg transition-colors"
      >
        <X size={20} class="text-gray-400" />
      </button>
    </div>

    <div class="p-6 max-h-[60vh] overflow-y-auto">
      <div class="mb-6">
        {#if isAdding}
          <div class="flex gap-2 p-4 bg-gray-800 rounded-xl border border-gray-700">
            <!-- svelte-ignore a11y-autofocus -->
            <input
              type="text"
              bind:value={newContainerName}
              placeholder="Enter container name..."
              class="flex-1 px-4 py-2 bg-gray-900 border-2 border-gray-600 rounded-lg text-white placeholder-gray-600 focus:outline-none focus:border-purple-500"
              on:keydown={(e) => e.key === 'Enter' && handleAddContainer()}
              autofocus
            />
            <button
              type="button"
              on:click={handleAddContainer}
              class="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-medium transition-colors"
            >
              Add
            </button>
            <button
              type="button"
              on:click={() => (isAdding = false, newContainerName = '')}
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium transition-colors"
            >
              Cancel
            </button>
          </div>
        {:else}
          <button
            type="button"
            on:click={() => (isAdding = true)}
            class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-purple-600 hover:bg-purple-700 text-white rounded-xl font-medium transition-colors shadow-lg shadow-purple-600/20"
          >
            <Plus size={18} />
            Create New Container
          </button>
        {/if}
      </div>

      <div class="space-y-2">
        <h3 class="text-sm font-semibold text-gray-400 mb-3 uppercase tracking-wider">All Containers ({containers.length})</h3>
        {#each containers as container}
          <div class="flex items-center justify-between p-4 bg-gray-800 rounded-xl hover:bg-gray-750 transition-all duration-200 border border-gray-700 group">
            <div class="flex items-center gap-3 flex-1 min-w-0">
              <div class="flex-shrink-0">
                <div class="p-2 rounded-lg {container.is_default ? 'bg-green-500/20' : 'bg-purple-500/20'}">
                  <Wallet size={18} class="{container.is_default ? 'text-green-400' : 'text-purple-400'}" />
                </div>
              </div>
              
              <div class="flex-1 min-w-0">
                {#if editingId === container.id}
                  <div class="flex items-center gap-2">
                    <!-- svelte-ignore a11y-autofocus -->
                    <input
                      type="text"
                      bind:value={editingName}
                      class="flex-1 px-3 py-1.5 bg-gray-900 border-2 border-gray-600 rounded-lg text-white text-sm focus:outline-none focus:border-purple-500"
                      on:keydown={(e) => {
                        if (e.key === 'Enter') saveEdit(container.id);
                        if (e.key === 'Escape') cancelEdit();
                      }}
                      autofocus
                    />
                    <button
                      on:click={() => saveEdit(container.id)}
                      class="p-1.5 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
                      title="Save"
                    >
                      <Check size={14} />
                    </button>
                    <button
                      on:click={cancelEdit}
                      class="p-1.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
                      title="Cancel"
                    >
                      <X size={14} />
                    </button>
                  </div>
                {:else}
                  <div class="flex items-center gap-2">
                    <span class="text-white font-semibold truncate">{container.name}</span>
                    {#if container.is_default}
                      <span class="text-xs px-2 py-0.5 bg-green-500/20 text-green-400 rounded-full flex-shrink-0">Default</span>
                    {/if}
                  </div>
                  <p class="text-xs text-gray-500 mt-0.5">Created {formatDate(container.created_at)}</p>
                {/if}
              </div>
            </div>

            {#if editingId !== container.id}
              <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  type="button"
                  on:click={() => startEditing(container)}
                  class="p-2 text-blue-400 hover:text-blue-300 hover:bg-blue-500/10 rounded-lg transition-all"
                  title="Rename container"
                >
                  <Edit2 size={16} />
                </button>
                {#if !container.is_default}
                  <button
                    type="button"
                    on:click={() => handleDeleteContainer(container)}
                    class="p-2 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-lg transition-all"
                    title="Delete container"
                  >
                    <Trash2 size={16} />
                  </button>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <div class="px-6 py-4 border-t border-gray-800 bg-gray-900/50">
      <div class="flex items-start gap-2">
        <div class="flex-shrink-0 mt-0.5">
          <div class="w-1.5 h-1.5 rounded-full bg-blue-400"></div>
        </div>
        <p class="text-xs text-gray-500">
          <strong class="text-gray-400">Containers</strong> let you manage separate balances in complete isolation. 
          Perfect for tracking personal vs business expenses, or different accounts. 
          Deleting a container will permanently remove all its transactions.
        </p>
      </div>
    </div>
  </div>
</div>

{#if errorMessage}
  <div class="fixed bottom-6 left-1/2 -translate-x-1/2 z-[60] bg-red-600 text-white px-6 py-3 rounded-xl shadow-lg flex items-center gap-3" in:fade={{ duration: 150 }}>
    <span class="text-sm">{errorMessage}</span>
    <button on:click={() => (errorMessage = '')} class="text-white/80 hover:text-white">
      <X size={16} />
    </button>
  </div>
{/if}

{#if pendingDeleteContainer}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[60] p-4" in:fade={{ duration: 150 }}>
    <div class="bg-gray-900 rounded-xl w-full max-w-sm border border-gray-700 shadow-2xl p-6 space-y-4">
      <h3 class="text-lg font-bold text-white">Delete Container</h3>
      <p class="text-sm text-gray-400">Delete container "{pendingDeleteContainer.name}"?</p>
      <p class="text-xs text-red-400">This will permanently delete all transactions in this container. This action cannot be undone.</p>
      <div class="flex gap-3 pt-2">
        <button
          on:click={confirmDeleteContainer}
          class="flex-1 bg-red-600 hover:bg-red-700 text-white px-4 py-2.5 rounded-lg font-semibold transition-colors"
        >
          Delete
        </button>
        <button
          on:click={() => (pendingDeleteContainer = null)}
          class="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2.5 rounded-lg font-semibold transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
