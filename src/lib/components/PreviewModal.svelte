<script lang="ts">
    import type { Tag, Wallpaper } from "$lib/types";

    interface Props {
        wallpaper: Wallpaper;
        tags: Tag[];
        loadingTags: boolean;
        onclose: () => void;
        onapply: (wp: Wallpaper) => void;
        onsearchtag: (tag: Tag) => void;
        onsearchsimilar: (wp: Wallpaper) => void;
    }

    let { wallpaper, tags, loadingTags, onclose, onapply, onsearchtag, onsearchsimilar }: Props = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="fixed inset-0 bg-black/80 backdrop-blur-[16px] flex items-center justify-center z-[100] animate-[fadeIn_0.2s_ease-out]"
    onclick={onclose}
    onkeydown={(e) => { if (e.key === 'Escape') onclose(); }}
>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
        class="relative max-w-[90%] max-h-[90%] flex flex-col items-center gap-3 animate-[fadeIn_0.25s_ease-out]"
        onclick={(e) => e.stopPropagation()}
    >
        <button
            class="absolute -top-2 -right-2 w-7 h-7 flex items-center justify-center rounded-full border-0 p-0 cursor-pointer z-10 bg-white/10 backdrop-blur-[8px] text-white hover:bg-white/25 hover:scale-110 active:scale-95 transition-all duration-200"
            onclick={onclose}
            title="Close"
        >
            <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
        </button>

        <img
            class="max-w-[480px] max-h-[70vh] rounded-xl object-contain shadow-[0_8px_40px_rgba(0,0,0,0.5)]"
            src={wallpaper.thumbs.large}
            alt={wallpaper.id}
        />

        {#if loadingTags}
            <p class="text-[11px] text-base-content/30 animate-pulse">Loading tags...</p>
        {:else if tags.length > 0}
            <div class="flex flex-wrap gap-1 max-w-[480px] justify-center">
                {#each tags as tag (tag.id)}
                    <button
                        class="px-3 py-1 rounded-full border border-white/10 bg-white/[0.06] text-base-content/55 text-[11px] cursor-pointer transition-all duration-200 hover:bg-primary/15 hover:border-primary/40 hover:text-primary-content hover:scale-105"
                        onclick={() => onsearchtag(tag)}
                    >{tag.name}</button>
                {/each}
            </div>
        {/if}

        <div class="flex items-center gap-2.5">
            <span class="text-base-content/40 text-xs font-medium">{wallpaper.resolution}</span>
            <button
                class="w-9 h-9 flex items-center justify-center rounded-full border border-white/[0.12] bg-white/[0.04] text-base-content/55 cursor-pointer p-0 hover:bg-white/10 hover:border-white/25 hover:scale-110 active:scale-95 transition-all duration-200"
                onclick={() => onsearchsimilar(wallpaper)}
                title="Find Similar"
            >
                <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"/></svg>
            </button>
            <button
                class="w-9 h-9 flex items-center justify-center rounded-full border-0 bg-primary text-primary-content cursor-pointer p-0 shadow-[0_2px_12px_oklch(var(--p)/0.3)] hover:scale-110 hover:shadow-[0_4px_20px_oklch(var(--p)/0.5)] active:scale-95 transition-all duration-200"
                onclick={() => onapply(wallpaper)}
                title="Apply Wallpaper"
            >
                <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
            </button>
        </div>
    </div>
</div>

<style>
    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }
</style>
