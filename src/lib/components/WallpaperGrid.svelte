<script lang="ts">
    import type { Wallpaper, View } from "$lib/types";

    interface Props {
        wallpapers: Wallpaper[];
        loading: boolean;
        loadingMore: boolean;
        error: string;
        hasMore: boolean;
        activeView: View;
        queue: Wallpaper[];
        settingWallpaper: string;
        onapply: (wp: Wallpaper) => void;
        onopenpreview: (wp: Wallpaper) => void;
        ontogglequeue: (wp: Wallpaper) => void;
        ondeletehistory: (wp: Wallpaper) => void;
    }

    let {
        wallpapers,
        loading,
        loadingMore,
        error,
        hasMore,
        activeView,
        queue,
        settingWallpaper,
        onapply,
        onopenpreview,
        ontogglequeue,
        ondeletehistory,
    }: Props = $props();
</script>

{#if loading}
    <div class="flex justify-center py-8">
        <span class="loading loading-spinner loading-md text-primary"></span>
    </div>

{:else if error}
    <p class="text-center py-6 text-error text-xs">{error}</p>

{:else if wallpapers.length === 0}
    <p class="text-center py-6 text-base-content/40 text-xs">No wallpapers found</p>

{:else}
    <div class="grid grid-cols-3 gap-1.5 p-1.5">
        {#each wallpapers as wp (wp.id)}
            <div
                class="thumb-wrapper group relative overflow-hidden rounded-lg transition-all duration-[250ms] ease-[cubic-bezier(0.34,1.56,0.64,1)]"
                class:setting={settingWallpaper === wp.id}
            >
                <button
                    class="thumb w-full border-0 p-0 bg-transparent block cursor-pointer disabled:cursor-default disabled:opacity-50"
                    onclick={() => onapply(wp)}
                    disabled={settingWallpaper !== ""}
                >
                    <img src={wp.thumbs.small} alt={wp.id} class="w-full block transition-[filter] duration-300" />
                    {#if settingWallpaper === wp.id}
                        <span class="absolute inset-0 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                            <span class="loading loading-spinner loading-sm text-white"></span>
                        </span>
                    {/if}
                </button>

                <!-- Preview icon (top-right) -->
                <button
                    class="absolute top-1.5 right-1.5 w-[26px] h-[26px] flex items-center justify-center rounded-lg p-0 cursor-pointer border-0 bg-black/50 backdrop-blur-[8px] text-white/85 opacity-0 -translate-y-1 group-hover:opacity-100 group-hover:translate-y-0 hover:bg-primary/60 hover:text-white transition-all duration-200"
                    onclick={() => onopenpreview(wp)}
                    title="Preview"
                >
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/></svg>
                </button>

                <!-- Queue icon (bottom-left) -->
                <button
                    class="queue-icon absolute bottom-1.5 left-1.5 w-[26px] h-[26px] flex items-center justify-center rounded-lg p-0 cursor-pointer border-0 bg-black/50 backdrop-blur-[8px] text-white/85 transition-all duration-200"
                    class:in-queue={queue.some((q) => q.id === wp.id)}
                    onclick={(e) => { e.stopPropagation(); ontogglequeue(wp); }}
                    title={queue.some((q) => q.id === wp.id) ? "Remove from queue" : "Add to queue"}
                >
                    {#if queue.some((q) => q.id === wp.id)}
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
                    {:else}
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M14 10H2v2h12v-2zm0-4H2v2h12V6zm4 8v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zM2 16h8v-2H2v2z"/></svg>
                    {/if}
                </button>

                <!-- Delete icon (history view only, top-left) -->
                {#if activeView.kind === "history"}
                    <button
                        class="absolute top-1.5 left-1.5 w-[26px] h-[26px] flex items-center justify-center rounded-lg p-0 cursor-pointer border-0 bg-black/50 backdrop-blur-[8px] text-error opacity-0 -translate-y-1 group-hover:opacity-100 group-hover:translate-y-0 hover:bg-error/70 hover:text-white transition-all duration-200"
                        onclick={(e) => { e.stopPropagation(); ondeletehistory(wp); }}
                        title="Remove from history"
                    >
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
                    </button>
                {/if}
            </div>
        {/each}
    </div>
    {#if loadingMore}
        <div class="flex justify-center py-3">
            <span class="loading loading-spinner loading-sm text-primary"></span>
        </div>
    {/if}
{/if}

<style>
    .thumb-wrapper:hover {
        transform: scale(1.03);
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
        z-index: 1;
    }

    .thumb-wrapper:hover .thumb img {
        filter: brightness(1.1) saturate(1.15);
    }

    .queue-icon {
        opacity: 0;
        transform: translateY(4px);
    }

    .queue-icon.in-queue {
        background: rgba(60, 170, 100, 0.7);
        color: #fff;
        opacity: 1;
        transform: translateY(0);
    }

    .thumb-wrapper:hover .queue-icon {
        opacity: 1;
        transform: translateY(0);
    }
</style>
