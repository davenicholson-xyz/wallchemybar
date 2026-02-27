<script lang="ts">
    import { openUrl } from "@tauri-apps/plugin-opener";
    import type { Tag, Wallpaper } from "$lib/types";

    interface Props {
        wallpaper: Wallpaper | null;
        tags: Tag[];
        loadingTags: boolean;
        onapply: (wp: Wallpaper) => void;
        onsearchtag: (tag: Tag) => void;
        onsearchsimilar: (wp: Wallpaper) => void;
        onloadtags?: () => void;
    }

    let { wallpaper, tags, loadingTags, onapply, onsearchtag, onsearchsimilar, onloadtags }: Props = $props();
</script>

<aside class="w-[280px] flex-shrink-0 flex flex-col bg-base-200/60 border-l border-base-300/50 overflow-y-auto panel" class:has-wallpaper={wallpaper !== null}>
    {#if wallpaper}
        <div class="flex flex-col gap-3 p-3">
            <img
                src={wallpaper.thumbs.large}
                alt={wallpaper.id}
                class="w-full rounded-lg object-cover shadow-[0_4px_20px_rgba(0,0,0,0.4)]"
            />

            <span class="text-xs text-base-content/35 font-medium px-1">{wallpaper.resolution}</span>

            {#if loadingTags}
                <p class="text-[11px] text-base-content/30 animate-pulse px-1">Loading tags...</p>
            {:else if tags.length > 0}
                <div class="flex flex-wrap gap-1">
                    {#each tags as tag (tag.id)}
                        <button
                            class="px-2.5 py-0.5 rounded-full border border-white/10 bg-white/[0.05] text-base-content/50 text-[11px] cursor-pointer transition-all duration-150 hover:bg-primary/15 hover:border-primary/40 hover:text-primary-content"
                            onclick={() => onsearchtag(tag)}
                        >{tag.name}</button>
                    {/each}
                </div>
            {:else if onloadtags}
                <button class="border-0 bg-transparent cursor-pointer px-1 text-left" onclick={onloadtags}>
                    <span class="text-[11px] text-base-content/40 hover:text-primary/80 underline underline-offset-2 decoration-base-content/20 transition-colors duration-150">Load tags</span>
                </button>
            {/if}

            <div class="flex flex-col gap-0.5">
                <button class="action-btn" onclick={() => onsearchsimilar(wallpaper!)} title="Find Similar">
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"/></svg>
                    Similar
                </button>
                <button class="action-btn" onclick={() => openUrl(`https://wallhaven.cc/w/${wallpaper!.id}`)} title="View on Wallhaven">
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M19 19H5V5h7V3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/></svg>
                    Wallhaven
                </button>
                <button class="action-btn apply" onclick={() => onapply(wallpaper!)} title="Apply Wallpaper">
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
                    Apply
                </button>
            </div>
        </div>
    {:else}
        <div class="flex flex-col items-center justify-center flex-1 gap-2 p-6 text-center">
            <svg viewBox="0 0 24 24" width="32" height="32" fill="currentColor" class="text-base-content/15">
                <path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"/>
            </svg>
            <p class="text-xs text-base-content/25">Hover or select a wallpaper<br>to preview it</p>
        </div>
    {/if}
</aside>

<style>
    .panel {
        opacity: 0.4;
        transition: opacity 0.2s ease;
    }
    .panel.has-wallpaper {
        opacity: 1;
    }

    .action-btn {
        width: 100%;
        height: 32px;
        border: none;
        border-radius: 6px;
        background: transparent;
        color: oklch(var(--bc) / 0.35);
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 0 8px;
        font-size: 12px;
        font-weight: 500;
        transition: background 0.15s ease-out, color 0.15s ease-out;
    }

    .action-btn:hover {
        background: rgba(255, 255, 255, 0.06);
        color: oklch(var(--bc) / 0.75);
    }

    .action-btn:active {
        transform: scale(0.98);
    }

    .action-btn.apply {
        color: oklch(var(--p) / 0.75);
    }

    .action-btn.apply:hover {
        background: oklch(var(--p) / 0.12);
        color: oklch(var(--p) / 0.9);
    }
</style>
