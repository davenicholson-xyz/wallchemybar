<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    interface Wallpaper {
        id: string;
        url: string;
        path: string;
        thumbs: { large: string; original: string; small: string };
        resolution: string;
    }

    interface Collection {
        id: number;
        label: string;
        views: number;
        public: number;
        count: number;
    }

    type View =
        | { kind: "search"; sorting: string }
        | { kind: "collection"; id: number };

    const categories: { label: string; sorting: string; icon: string }[] = [
        {
            label: "Latest",
            sorting: "date_added",
            icon: "M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.4 0-8-3.6-8-8s3.6-8 8-8 8 3.6 8 8-3.6 8-8 8zm.5-13H11v6l5.2 3.1.8-1.3-4.5-2.7V7z",
        },
        {
            label: "Hot",
            sorting: "hot",
            icon: "M13.5 0.67s.74 2.65.74 4.8c0 2.06-1.35 3.73-3.41 3.73-2.07 0-3.63-1.67-3.63-3.73l.03-.36C5.21 7.51 4 10.62 4 14c0 4.42 3.58 8 8 8s8-3.58 8-8C20 8.61 17.41 3.8 13.5.67zM11.71 19c-1.78 0-3.22-1.4-3.22-3.14 0-1.62 1.05-2.76 2.81-3.12 1.77-.36 3.6-1.21 4.62-2.58.39 1.29.59 2.65.59 4.04 0 2.65-2.15 4.8-4.8 4.8z",
        },
        {
            label: "Top",
            sorting: "toplist",
            icon: "M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z",
        },
        {
            label: "Random",
            sorting: "random",
            icon: "M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z",
        },
    ];

    let collections: Collection[] = $state([]);
    let activeView: View = $state({ kind: "search", sorting: "hot" });
    let wallpapers: Wallpaper[] = $state([]);
    let loading = $state(true);
    let loadingMore = $state(false);
    let error = $state("");
    let settingWallpaper = $state("");
    let page = $state(1);
    let hasMore = $state(true);
    let mainEl: HTMLElement;

    onMount(async () => {
        invoke("fetch_collections")
            .then((cols) => {
                collections = cols as Collection[];
            })
            .catch(() => {});

        await loadSearch("hot");
    });

    async function loadSearch(sorting: string) {
        activeView = { kind: "search", sorting };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        try {
            const results: Wallpaper[] = await invoke("fetch_search", {
                sorting,
                page: 1,
            });
            wallpapers = results;
            hasMore = results.length >= 24;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    async function loadCollection(id: number) {
        activeView = { kind: "collection", id };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        try {
            const results: Wallpaper[] = await invoke(
                "fetch_collection_wallpapers",
                { collectionId: id, page: 1 },
            );
            wallpapers = results;
            hasMore = results.length >= 24;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    async function loadNextPage() {
        if (loadingMore || !hasMore) return;
        loadingMore = true;
        const nextPage = page + 1;
        try {
            let results: Wallpaper[];
            if (activeView.kind === "search") {
                results = await invoke("fetch_search", {
                    sorting: activeView.sorting,
                    page: nextPage,
                });
            } else {
                results = await invoke("fetch_collection_wallpapers", {
                    collectionId: activeView.id,
                    page: nextPage,
                });
            }
            wallpapers = [...wallpapers, ...results];
            page = nextPage;
            hasMore = results.length >= 24;
        } catch (e) {
            error = String(e);
        } finally {
            loadingMore = false;
        }
    }

    function onScroll() {
        if (!mainEl) return;
        const { scrollTop, scrollHeight, clientHeight } = mainEl;
        if (scrollHeight - scrollTop - clientHeight < 200) {
            loadNextPage();
        }
    }

    async function applyWallpaper(wp: Wallpaper) {
        settingWallpaper = wp.id;
        try {
            await invoke("set_wallpaper", { imageUrl: wp.path });
        } catch (e) {
            error = String(e);
        } finally {
            settingWallpaper = "";
        }
    }

    function isActive(cat: { sorting: string }): boolean {
        return (
            activeView.kind === "search" && activeView.sorting === cat.sorting
        );
    }

    function isCollectionActive(id: number): boolean {
        return activeView.kind === "collection" && activeView.id === id;
    }
</script>

<div class="layout">
    <nav class="sidebar">
        {#each categories as cat}
            <button
                class="nav-btn"
                class:active={isActive(cat)}
                title={cat.label}
                onclick={() => loadSearch(cat.sorting)}
            >
                <svg
                    viewBox="0 0 24 24"
                    width="20"
                    height="20"
                    fill="currentColor"
                >
                    <path d={cat.icon} />
                </svg>
            </button>
        {/each}

        {#if collections.length > 0}
            <div class="divider"></div>
            {#each collections as col (col.id)}
                <button
                    class="nav-btn"
                    class:active={isCollectionActive(col.id)}
                    title="{col.label} ({col.count})"
                    onclick={() => loadCollection(col.id)}
                >
                    <svg
                        viewBox="0 0 24 24"
                        width="20"
                        height="20"
                        fill="currentColor"
                    >
                        <path
                            d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"
                        />
                    </svg>
                </button>
            {/each}
        {/if}
    </nav>

    <main bind:this={mainEl} onscroll={onScroll}>
        {#if loading}
            <p class="status">Loading...</p>
        {:else if error}
            <p class="status error">{error}</p>
        {:else if wallpapers.length === 0}
            <p class="status">No wallpapers found</p>
        {:else}
            <div class="grid">
                {#each wallpapers as wp (wp.id)}
                    <button
                        class="thumb"
                        class:setting={settingWallpaper === wp.id}
                        onclick={() => applyWallpaper(wp)}
                        disabled={settingWallpaper !== ""}
                    >
                        <img src={wp.thumbs.small} alt={wp.id} />
                        <span class="res">{wp.resolution}</span>
                        {#if settingWallpaper === wp.id}
                            <span class="applying">...</span>
                        {/if}
                    </button>
                {/each}
            </div>
            {#if loadingMore}
                <p class="status">Loading more...</p>
            {/if}
        {/if}
    </main>
</div>

<style>
    :root {
        margin: 0;
        padding: 0;
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        font-size: 12px;
        color: #f6f6f6;
        background-color: #1a1a1a;
    }

    .layout {
        display: flex;
        height: 100vh;
        width: 100%;
    }

    .sidebar {
        width: 44px;
        flex-shrink: 0;
        background: #111;
        border-right: 1px solid #333;
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 8px 0;
        gap: 4px;
        overflow-y: auto;
    }

    .nav-btn {
        width: 34px;
        height: 34px;
        border: none;
        border-radius: 6px;
        background: transparent;
        color: #777;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        transition:
            background 0.15s,
            color 0.15s;
    }

    .nav-btn:hover {
        background: #2a2a2a;
        color: #ccc;
    }

    .nav-btn.active {
        background: #333;
        color: #fff;
    }

    .divider {
        width: 24px;
        height: 1px;
        background: #333;
        margin: 4px 0;
    }

    main {
        flex: 1;
        overflow-y: auto;
        min-width: 0;
    }

    .status {
        text-align: center;
        padding: 20px;
        color: #999;
    }

    .error {
        color: #ff6b6b;
    }

    .grid {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        gap: 4px;
        padding: 4px;
    }

    .thumb {
        position: relative;
        overflow: hidden;
        border-radius: 4px;
        cursor: pointer;
        border: none;
        padding: 0;
        background: none;
    }

    .thumb img {
        width: 100%;
        display: block;
    }

    .thumb:hover:not(:disabled) {
        opacity: 0.85;
    }

    .thumb:disabled {
        cursor: default;
        opacity: 0.5;
    }

    .thumb.setting {
        opacity: 1;
    }

    .applying {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0, 0, 0, 0.5);
        color: #fff;
        font-size: 14px;
        font-weight: bold;
    }

    .res {
        position: absolute;
        bottom: 2px;
        right: 4px;
        font-size: 10px;
        color: #fff;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
    }
</style>
