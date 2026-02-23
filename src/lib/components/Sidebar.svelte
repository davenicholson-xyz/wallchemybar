<script lang="ts">
    import type { Collection, View } from "$lib/types";

    interface Category {
        label: string;
        sorting: string;
        icon: string;
    }

    interface Props {
        activeView: View;
        collections: Collection[];
        undoing: boolean;
        categories: Category[];
        isactive: (cat: Category) => boolean;
        onloadsearch: (sorting: string) => void;
        onloadhistory: () => void;
        onactivatequeue: () => void;
        onactivatesearch: () => void;
        onactivatecollections: () => void;
        onopensettings: () => void;
        onundowallpaper: () => void;
    }

    let {
        activeView,
        collections,
        undoing,
        categories,
        isactive,
        onloadsearch,
        onloadhistory,
        onactivatequeue,
        onactivatesearch,
        onactivatecollections,
        onopensettings,
        onundowallpaper,
    }: Props = $props();

    const sortingKey: Record<string, string> = {
        date_added: "N",
        toplist: "T",
        random: "R",
    };
</script>

<nav class="w-[46px] flex-shrink-0 flex flex-col items-center py-2.5 pb-5 gap-1 bg-base-200 shadow-[2px_0_12px_rgba(0,0,0,0.4)] z-10 overflow-y-auto">
    <button class="nav-btn" class:active={activeView.kind === "query"} title="Search (S)" onclick={onactivatesearch}>
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
        </svg>
    </button>
    <button class="nav-btn" class:active={activeView.kind === "history"} title="History (H)" onclick={onloadhistory}>
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.4 0-8-3.6-8-8s3.6-8 8-8 8 3.6 8 8-3.6 8-8 8zm.5-13H11v6l5.2 3.1.8-1.3-4.5-2.7V7z"/>
        </svg>
    </button>
    <button class="nav-btn" class:active={activeView.kind === "queue"} title="Queue (Q)" onclick={onactivatequeue}>
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z"/>
        </svg>
    </button>

    <div class="w-[22px] h-px bg-gradient-to-r from-transparent via-base-content/10 to-transparent my-1"></div>

    {#each categories as cat}
        <button class="nav-btn" class:active={isactive(cat)} title={sortingKey[cat.sorting] ? `${cat.label} (${sortingKey[cat.sorting]})` : cat.label} onclick={() => onloadsearch(cat.sorting)}>
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d={cat.icon}/>
            </svg>
        </button>
    {/each}

    {#if collections.length > 0}
        <div class="w-[22px] h-px bg-gradient-to-r from-transparent via-base-content/10 to-transparent my-1"></div>
        <button
            class="nav-btn"
            class:active={activeView.kind === "collections" || activeView.kind === "collection"}
            title="Collections (C)"
            onclick={onactivatecollections}
        >
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
            </svg>
        </button>
    {/if}

    <div class="flex-1"></div>

    <button class="nav-btn" title="Undo (U)" onclick={onundowallpaper} disabled={undoing}>
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M12.5 8c-2.65 0-5.05.99-6.9 2.6L2 7v9h9l-3.62-3.62c1.39-1.16 3.16-1.88 5.12-1.88 3.54 0 6.55 2.31 7.6 5.5l2.37-.78C21.08 11.03 17.15 8 12.5 8z"/>
        </svg>
    </button>
    <button class="nav-btn" class:active={activeView.kind === "settings"} title="Settings (,)" onclick={onopensettings}>
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.49.49 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.49.49 0 00-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6A3.6 3.6 0 1112 8.4a3.6 3.6 0 010 7.2z"/>
        </svg>
    </button>
</nav>

<style>
    .nav-btn {
        position: relative;
        width: 34px;
        height: 34px;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: oklch(var(--bc) / 0.3);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        transition: background 0.2s ease-out, color 0.2s ease-out,
            transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.2s ease-out;
    }

    .nav-btn:hover {
        background: rgba(255, 255, 255, 0.06);
        color: oklch(var(--bc) / 0.7);
        transform: scale(1.08);
    }

    .nav-btn:active {
        transform: scale(0.95);
    }

    .nav-btn:disabled {
        opacity: 0.35;
        cursor: default;
        transform: none !important;
    }

    .nav-btn.active {
        background: oklch(var(--p) / 0.12);
        color: oklch(var(--p) / 0.85);
        box-shadow: 0 0 12px oklch(var(--p) / 0.15);
    }

    .nav-btn.active::before {
        content: '';
        position: absolute;
        left: -6px;
        top: 50%;
        transform: translateY(-50%);
        width: 3px;
        height: 18px;
        background: oklch(var(--p));
        border-radius: 0 3px 3px 0;
    }
</style>
