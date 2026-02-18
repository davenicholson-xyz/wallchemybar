<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    interface Tag {
        id: number;
        name: string;
    }

    interface Wallpaper {
        id: string;
        url: string;
        path: string;
        thumbs: { large: string; original: string; small: string };
        resolution: string;
        tags?: Tag[];
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
        | { kind: "collections" }
        | { kind: "collection"; id: number }
        | { kind: "query"; query: string }
        | { kind: "history" }
        | { kind: "queue" }
        | { kind: "settings" };

    const categories: { label: string; sorting: string; icon: string }[] = [
        {
            label: "Latest",
            sorting: "date_added",
            icon: "M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5v-3h3.56c.69 1.19 1.97 2 3.45 2s2.75-.81 3.45-2H19v3zm0-5h-4.99c0 1.1-.9 2-2 2s-2-.9-2-2H5V5h14v9z",
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
    let searchInput = $state("");
    let selectedCollectionId: number | null = $state(null);
    let undoing = $state(false);
    let previewWallpaper: Wallpaper | null = $state(null);
    let previewTags: Tag[] = $state([]);
    let loadingTags = $state(false);

    // Queue state
    let queue: Wallpaper[] = $state([]);
    let queueIntervalMinutes = $state(30);
    let queueRunning = $state(false);
    let queueIndex = $state(0);
    let queueTimerId: ReturnType<typeof setInterval> | undefined;
    let dragFromIndex: number | null = $state(null);
    let dragOverIndex: number | null = $state(null);
    let queueDragging = $state(false);

    // Settings state
    let settingsUsername = $state("");
    let settingsApiKey = $state("");
    let settingsSfw = $state(true);
    let settingsSketchy = $state(false);
    let settingsNsfw = $state(false);
    let settingsGeneral = $state(true);
    let settingsAnime = $state(true);
    let settingsPeople = $state(true);
    let settingsAtleast = $state("");
    const availableResolutions = [
        "1280x720", "1280x800", "1280x960", "1280x1024",
        "1600x900", "1600x1000", "1600x1200", "1600x1280",
        "1920x1080", "1920x1200", "1920x1440", "1920x1536",
        "2560x1080", "2560x1440", "2560x1600", "2560x1920", "2560x2048",
        "3440x1440", "3840x1600", "3840x2160", "3840x2400", "3840x2880", "3840x3072",
    ];

    async function undoWallpaper() {
        undoing = true;
        try {
            await invoke("undo_wallpaper");
            invoke("hide_main");
        } catch (e) {
            error = String(e);
        } finally {
            undoing = false;
        }
    }

    onMount(async () => {
        invoke("fetch_collections")
            .then((cols) => {
                collections = cols as Collection[];
            })
            .catch(() => {});

        await loadSearch("hot");
    });

    async function loadSearch(sorting: string) {
        console.log(`[wallchemybar] loadSearch: sorting=${sorting}`);
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
            console.log(`[wallchemybar] loadSearch: got ${results.length} results, hasMore=${hasMore}`);
        } catch (e) {
            console.error("[wallchemybar] loadSearch: error:", e);
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

    function activateCollections() {
        if (collections.length > 0) {
            selectCollection(collections[0].id);
        } else {
            activeView = { kind: "collections" };
            selectedCollectionId = null;
            wallpapers = [];
            loading = false;
            error = "";
        }
    }

    async function selectCollection(id: number) {
        selectedCollectionId = id;
        await loadCollection(id);
    }

    function activateSearch() {
        activeView = { kind: "query", query: "" };
        wallpapers = [];
        loading = false;
        error = "";
    }

    async function searchQuery() {
        const q = searchInput.trim();
        if (!q) return;
        activeView = { kind: "query", query: q };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        try {
            const results: Wallpaper[] = await invoke("fetch_search", {
                sorting: "relevance",
                page: 1,
                query: q,
            });
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
        if (activeView.kind === "history") return;
        loadingMore = true;
        const nextPage = page + 1;
        const startTime = performance.now();
        console.log(`[wallchemybar] loadNextPage: requesting page ${nextPage}, view=${activeView.kind}, current total=${wallpapers.length}`);
        try {
            let results: Wallpaper[];
            if (activeView.kind === "search") {
                results = await invoke("fetch_search", {
                    sorting: activeView.sorting,
                    page: nextPage,
                });
            } else if (activeView.kind === "query") {
                results = await invoke("fetch_search", {
                    sorting: "relevance",
                    page: nextPage,
                    query: activeView.query,
                });
            } else if (activeView.kind === "collection") {
                results = await invoke("fetch_collection_wallpapers", {
                    collectionId: activeView.id,
                    page: nextPage,
                });
            } else {
                console.warn(`[wallchemybar] loadNextPage: unexpected view kind: ${activeView.kind}`);
                loadingMore = false;
                return;
            }
            const elapsed = (performance.now() - startTime).toFixed(0);
            const existingIds = new Set(wallpapers.map((w) => w.id));
            const newResults = results.filter((w) => !existingIds.has(w.id));
            const dupes = results.length - newResults.length;
            if (dupes > 0) {
                console.warn(`[wallchemybar] loadNextPage: filtered ${dupes} duplicate wallpapers from page ${nextPage}`);
            }
            console.log(`[wallchemybar] loadNextPage: got ${results.length} results (${newResults.length} new) for page ${nextPage} in ${elapsed}ms`);
            wallpapers = [...wallpapers, ...newResults];
            page = nextPage;
            hasMore = results.length >= 24;
            console.log(`[wallchemybar] loadNextPage: hasMore=${hasMore}, total wallpapers=${wallpapers.length}`);
        } catch (e) {
            console.error(`[wallchemybar] loadNextPage: error on page ${nextPage}:`, e);
            // Don't wipe the grid — show the error alongside existing results
            hasMore = false;
        } finally {
            loadingMore = false;
        }
    }

    let scrollTicking = false;
    function onScroll() {
        if (scrollTicking) return;
        scrollTicking = true;
        requestAnimationFrame(() => {
            scrollTicking = false;
            if (!mainEl) return;
            const { scrollTop, scrollHeight, clientHeight } = mainEl;
            const remaining = scrollHeight - scrollTop - clientHeight;
            if (remaining < 200) {
                console.log(`[wallchemybar] onScroll: near bottom (${remaining.toFixed(0)}px remaining), loadingMore=${loadingMore}, hasMore=${hasMore}`);
                loadNextPage();
            }
        });
    }

    async function applyWallpaper(wp: Wallpaper) {
        invoke("hide_main");
        settingWallpaper = wp.id;
        try {
            await invoke("set_wallpaper", { wallpaper: wp });
        } catch (e) {
            error = String(e);
        } finally {
            settingWallpaper = "";
        }
    }

    async function loadHistory() {
        activeView = { kind: "history" };
        loading = true;
        error = "";
        hasMore = false;
        try {
            const entries: Wallpaper[] = await invoke("get_history");
            wallpapers = entries;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    async function deleteHistoryEntry(wp: Wallpaper) {
        try {
            await invoke("delete_history_entry", { wallpaperId: wp.id });
            wallpapers = wallpapers.filter((w) => w.id !== wp.id);
        } catch (e) {
            error = String(e);
        }
    }

    async function openSettings() {
        activeView = { kind: "settings" };
        loading = true;
        try {
            const settings: {
                username: string;
                api_key: string;
                purity: string;
                categories: string;
                atleast: string;
            } = await invoke("load_settings");
            settingsUsername = settings.username;
            settingsApiKey = settings.api_key;
            settingsSfw = settings.purity[0] === "1";
            settingsSketchy = settings.purity[1] === "1";
            settingsNsfw = settings.purity[2] === "1";
            const cats = settings.categories ?? "111";
            settingsGeneral = cats[0] === "1";
            settingsAnime = cats[1] === "1";
            settingsPeople = cats[2] === "1";
            settingsAtleast = settings.atleast ?? "";
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    async function saveSettings() {
        const purity = `${settingsSfw ? "1" : "0"}${settingsSketchy ? "1" : "0"}${settingsNsfw ? "1" : "0"}`;
        const categories = `${settingsGeneral ? "1" : "0"}${settingsAnime ? "1" : "0"}${settingsPeople ? "1" : "0"}`;
        await invoke("save_settings", {
            settings: {
                username: settingsUsername,
                api_key: settingsApiKey,
                purity,
                categories,
                atleast: settingsAtleast,
            },
        });
        await loadSearch("hot");
    }

    async function clearHistory() {
        await invoke("clear_history");
    }

    async function openPreview(wp: Wallpaper) {
        previewWallpaper = wp;
        previewTags = [];
        loadingTags = true;
        try {
            const tags: Tag[] = await invoke("fetch_wallpaper_tags", { wallpaperId: wp.id });
            if (previewWallpaper?.id === wp.id) {
                previewTags = tags;
            }
        } catch {
            // silently ignore — tags are non-critical
        } finally {
            loadingTags = false;
        }
    }

    async function searchTag(tag: Tag) {
        previewWallpaper = null;
        const q = tag.name;
        searchInput = q;
        activeView = { kind: "query", query: q };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        try {
            const results: Wallpaper[] = await invoke("fetch_search", {
                sorting: "relevance",
                page: 1,
                query: q,
            });
            wallpapers = results;
            hasMore = results.length >= 24;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    async function searchSimilar(wp: Wallpaper) {
        const q = "like:" + wp.id;
        previewWallpaper = null;
        searchInput = q;
        activeView = { kind: "query", query: q };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        try {
            const results: Wallpaper[] = await invoke("fetch_search", {
                sorting: "relevance",
                page: 1,
                query: q,
            });
            wallpapers = results;
            hasMore = results.length >= 24;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    function isActive(cat: { sorting: string }): boolean {
        return (
            activeView.kind === "search" && activeView.sorting === cat.sorting
        );
    }

    // ─── Queue ───────────────────────────────────────────────────────────────

    async function activateQueue() {
        activeView = { kind: "queue" };
        loading = true;
        error = "";
        try {
            queue = await invoke("get_queue");
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    async function toggleQueue(wp: Wallpaper) {
        const inQueue = queue.some((q) => q.id === wp.id);
        try {
            if (inQueue) {
                await invoke("remove_from_queue", { wallpaperId: wp.id });
            } else {
                await invoke("add_to_queue", { wallpaper: wp });
            }
            queue = await invoke("get_queue");
        } catch (e) {
            error = String(e);
        }
    }

    function startCycling() {
        if (queue.length === 0) return;
        queueRunning = true;
        queueTimerId = setInterval(advanceQueue, queueIntervalMinutes * 60 * 1000);
    }

    function stopCycling() {
        queueRunning = false;
        clearInterval(queueTimerId);
        queueTimerId = undefined;
    }

    async function advanceQueue() {
        const current = await invoke<Wallpaper[]>("get_queue");
        if (current.length === 0) { stopCycling(); return; }
        const idx = queueIndex % current.length;
        queueIndex = (idx + 1) % current.length;
        await applyWallpaper(current[idx]);
    }

    function changeQueueInterval(minutes: number) {
        queueIntervalMinutes = minutes;
        if (queueRunning) {
            clearInterval(queueTimerId);
            queueTimerId = setInterval(advanceQueue, minutes * 60 * 1000);
        }
    }

    async function removeFromQueue(wp: Wallpaper) {
        try {
            await invoke("remove_from_queue", { wallpaperId: wp.id });
            queue = await invoke("get_queue");
            if (queueIndex >= queue.length && queueIndex > 0) {
                queueIndex = queue.length - 1;
            }
            if (queue.length === 0) stopCycling();
        } catch (e) {
            error = String(e);
        }
    }

    async function clearQueue() {
        try {
            await invoke("clear_queue");
            queue = [];
            queueIndex = 0;
            stopCycling();
        } catch (e) {
            error = String(e);
        }
    }

    function onDragHandleMouseDown(e: MouseEvent, idx: number) {
        e.preventDefault();
        console.log(`[queue] drag start idx=${idx}`);
        dragFromIndex = idx;
        dragOverIndex = idx;
        queueDragging = true;
        window.addEventListener("mousemove", onQueueMouseMove);
        window.addEventListener("mouseup", onQueueMouseUp);
    }

    function onQueueMouseMove(e: MouseEvent) {
        const els = document.elementsFromPoint(e.clientX, e.clientY);
        for (const el of els) {
            const attr = (el as HTMLElement).dataset.queueIdx;
            if (attr !== undefined) {
                const idx = parseInt(attr);
                if (dragOverIndex !== idx) {
                    console.log(`[queue] hover over idx=${idx} (from=${dragFromIndex})`);
                    dragOverIndex = idx;
                }
                return;
            }
        }
    }

    function onQueueMouseUp() {
        window.removeEventListener("mousemove", onQueueMouseMove);
        window.removeEventListener("mouseup", onQueueMouseUp);
        console.log(`[queue] drop: from=${dragFromIndex} to=${dragOverIndex}`);
        if (dragFromIndex !== null && dragOverIndex !== null && dragFromIndex !== dragOverIndex) {
            const reordered = [...queue];
            const [moved] = reordered.splice(dragFromIndex, 1);
            reordered.splice(dragOverIndex, 0, moved);
            console.log(`[queue] reordered:`, reordered.map(w => w.id));
            queue = reordered;
            invoke("reorder_queue", { wallpapers: reordered }).catch(err => { error = String(err); });
        } else {
            console.log(`[queue] drop skipped (same position or no source)`);
        }
        dragFromIndex = null;
        dragOverIndex = null;
        queueDragging = false;
    }

    const intervalOptions = [
        { label: "5 min", value: 5 },
        { label: "10 min", value: 10 },
        { label: "15 min", value: 15 },
        { label: "30 min", value: 30 },
        { label: "1 hr", value: 60 },
        { label: "2 hr", value: 120 },
    ];
</script>

<div class="layout">
    <nav class="sidebar">
        <button
            class="nav-btn"
            class:active={activeView.kind === "query"}
            title="Search"
            onclick={activateSearch}
        >
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path
                    d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"
                />
            </svg>
        </button>
        <button
            class="nav-btn"
            class:active={activeView.kind === "history"}
            title="History"
            onclick={loadHistory}
        >
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path
                    d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.4 0-8-3.6-8-8s3.6-8 8-8 8 3.6 8 8-3.6 8-8 8zm.5-13H11v6l5.2 3.1.8-1.3-4.5-2.7V7z"
                />
            </svg>
        </button>
        <button
            class="nav-btn"
            class:active={activeView.kind === "queue"}
            title="Queue"
            onclick={activateQueue}
        >
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z"/>
            </svg>
        </button>
        <div class="divider"></div>
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
            <button
                class="nav-btn"
                class:active={activeView.kind === "collections" ||
                    activeView.kind === "collection"}
                title="Collections"
                onclick={activateCollections}
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
        {/if}
        <div class="spacer"></div>
        <button
            class="nav-btn"
            title="Undo"
            onclick={undoWallpaper}
            disabled={undoing}
        >
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path
                    d="M12.5 8c-2.65 0-5.05.99-6.9 2.6L2 7v9h9l-3.62-3.62c1.39-1.16 3.16-1.88 5.12-1.88 3.54 0 6.55 2.31 7.6 5.5l2.37-.78C21.08 11.03 17.15 8 12.5 8z"
                />
            </svg>
        </button>
        <button class="nav-btn" class:active={activeView.kind === "settings"} title="Settings" onclick={openSettings}>
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path
                    d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.49.49 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.49.49 0 00-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6A3.6 3.6 0 1112 8.4a3.6 3.6 0 010 7.2z"
                />
            </svg>
        </button>
    </nav>

    <main bind:this={mainEl} onscroll={onScroll}>
        {#if activeView.kind === "collections" || activeView.kind === "collection"}
            <div class="collection-picker">
                <select
                    value={selectedCollectionId ?? ""}
                    onchange={(e) => {
                        const val = (e.target as HTMLSelectElement).value;
                        if (val) selectCollection(Number(val));
                    }}
                >
                    <option value="" disabled>Select a collection...</option>
                    {#each collections as col (col.id)}
                        <option value={col.id}>{col.label} ({col.count})</option
                        >
                    {/each}
                </select>
            </div>
        {/if}
        {#if activeView.kind === "query"}
            <form
                class="search-bar"
                onsubmit={(e) => {
                    e.preventDefault();
                    searchQuery();
                }}
            >
                <input
                    type="text"
                    bind:value={searchInput}
                    placeholder="Search wallpapers..."
                />
            </form>
        {/if}
        {#if activeView.kind === "settings"}
            <div class="settings-panel">
                <h3 class="settings-title">Settings</h3>
                <form class="settings-form" onsubmit={(e) => { e.preventDefault(); saveSettings(); }}>
                    <label class="settings-label">
                        <span class="settings-label-text">Username</span>
                        <input type="text" class="settings-input" bind:value={settingsUsername} placeholder="wallhaven username" />
                    </label>
                    <label class="settings-label">
                        <span class="settings-label-text">API Key</span>
                        <input type="password" class="settings-input" bind:value={settingsApiKey} placeholder="wallhaven API key" />
                    </label>
                    <div class="settings-field">
                        <span class="settings-label-text">Categories</span>
                        <div class="settings-toggle-group">
                            <button type="button" class="settings-toggle category" class:active={settingsGeneral} onclick={() => (settingsGeneral = !settingsGeneral)}>General</button>
                            <button type="button" class="settings-toggle category" class:active={settingsAnime} onclick={() => (settingsAnime = !settingsAnime)}>Anime</button>
                            <button type="button" class="settings-toggle category" class:active={settingsPeople} onclick={() => (settingsPeople = !settingsPeople)}>People</button>
                        </div>
                    </div>
                    <div class="settings-field">
                        <span class="settings-label-text">Purity</span>
                        <div class="settings-toggle-group">
                            <button type="button" class="settings-toggle sfw" class:active={settingsSfw} onclick={() => (settingsSfw = !settingsSfw)}>SFW</button>
                            <button type="button" class="settings-toggle sketchy" class:active={settingsSketchy} onclick={() => (settingsSketchy = !settingsSketchy)}>Sketchy</button>
                            <button type="button" class="settings-toggle nsfw" class:active={settingsNsfw} onclick={() => (settingsNsfw = !settingsNsfw)}>NSFW</button>
                        </div>
                    </div>
                    <label class="settings-label">
                        <span class="settings-label-text">Minimum Resolution</span>
                        <select class="settings-input" bind:value={settingsAtleast}>
                            <option value="">Any</option>
                            {#each availableResolutions as res}
                                <option value={res}>{res}</option>
                            {/each}
                        </select>
                    </label>
                    <button class="settings-save" type="submit">Save</button>
                </form>
                <div class="settings-danger">
                    <button class="settings-clear-history" onclick={clearHistory}>Clear History</button>
                </div>
            </div>
        {:else if activeView.kind === "queue"}
            <div class="queue-view">
                <div class="queue-controls">
                    <button
                        class="queue-play-btn"
                        class:running={queueRunning}
                        onclick={() => queueRunning ? stopCycling() : startCycling()}
                        disabled={queue.length === 0}
                        title={queueRunning ? "Stop cycling" : "Start cycling"}
                    >
                        {#if queueRunning}
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
                            </svg>
                            Stop
                        {:else}
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                <path d="M8 5v14l11-7z"/>
                            </svg>
                            Start
                        {/if}
                    </button>
                    <div class="queue-interval">
                        <span class="queue-interval-label">Cycle every</span>
                        <div class="queue-interval-pills">
                            {#each intervalOptions as opt}
                                <button
                                    class="queue-interval-pill"
                                    class:active={queueIntervalMinutes === opt.value}
                                    onclick={() => changeQueueInterval(opt.value)}
                                >{opt.label}</button>
                            {/each}
                        </div>
                    </div>
                </div>
                {#if queue.length === 0}
                    <p class="status">Queue is empty — add wallpapers from search views</p>
                {:else}
                    <div class="grid" class:dragging={queueDragging}>
                        {#each queue as wp, idx (wp.id)}
                            <div
                                class="thumb-wrapper"
                                class:setting={settingWallpaper === wp.id}
                                class:queue-active={idx === queueIndex % queue.length && queueRunning}
                                class:drag-source={queueDragging && dragFromIndex === idx}
                                class:drag-over={dragOverIndex === idx && queueDragging && dragFromIndex !== idx}
                                data-queue-idx={idx}
                            >
                                <button
                                    class="thumb"
                                    onclick={() => applyWallpaper(wp)}
                                    disabled={settingWallpaper !== ""}
                                >
                                    <img src={wp.thumbs.small} alt={wp.id} />
                                    {#if settingWallpaper === wp.id}
                                        <span class="applying">...</span>
                                    {/if}
                                </button>
                                <button
                                    class="delete-icon"
                                    onclick={(e) => { e.stopPropagation(); removeFromQueue(wp); }}
                                    title="Remove from queue"
                                >
                                    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                        <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                                    </svg>
                                </button>
                                <!-- svelte-ignore a11y_no_static_element_interactions -->
                                <div
                                    class="drag-handle"
                                    title="Drag to reorder"
                                    onmousedown={(e) => onDragHandleMouseDown(e, idx)}
                                >
                                    <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                                        <path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/>
                                    </svg>
                                </div>
                            </div>
                        {/each}
                    </div>
                    <div class="queue-footer">
                        <button class="queue-clear-btn" onclick={clearQueue}>Clear Queue</button>
                    </div>
                {/if}
            </div>
        {:else if loading}
            <p class="status error">{error}</p>
        {:else if wallpapers.length === 0}
            <p class="status">No wallpapers found</p>
        {:else}
            <div class="grid">
                {#each wallpapers as wp (wp.id)}
                    <div
                        class="thumb-wrapper"
                        class:setting={settingWallpaper === wp.id}
                    >
                        <button
                            class="thumb"
                            onclick={() => applyWallpaper(wp)}
                            disabled={settingWallpaper !== ""}
                        >
                            <img src={wp.thumbs.small} alt={wp.id} />

                            {#if settingWallpaper === wp.id}
                                <span class="applying">...</span>
                            {/if}
                        </button>
                        <button
                            class="preview-icon"
                            onclick={() => openPreview(wp)}
                            title="Preview"
                        >
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                            </svg>
                        </button>
                        <button
                            class="queue-icon"
                            class:in-queue={queue.some((q) => q.id === wp.id)}
                            onclick={(e) => { e.stopPropagation(); toggleQueue(wp); }}
                            title={queue.some((q) => q.id === wp.id) ? "Remove from queue" : "Add to queue"}
                        >
                            {#if queue.some((q) => q.id === wp.id)}
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                                </svg>
                            {:else}
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                    <path d="M14 10H2v2h12v-2zm0-4H2v2h12V6zm4 8v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zM2 16h8v-2H2v2z"/>
                                </svg>
                            {/if}
                        </button>
                        {#if activeView.kind === "history"}
                            <button
                                class="delete-icon"
                                onclick={(e) => { e.stopPropagation(); deleteHistoryEntry(wp); }}
                                title="Remove from history"
                            >
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                                </svg>
                            </button>
                        {/if}
                    </div>
                {/each}
            </div>
            {#if loadingMore}
                <p class="status">Loading more...</p>
            {/if}
        {/if}
    </main>
</div>

{#if previewWallpaper}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="preview-overlay" onclick={() => (previewWallpaper = null)} onkeydown={(e) => { if (e.key === 'Escape') previewWallpaper = null; }}>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="preview-content" onclick={(e) => e.stopPropagation()}>
            <button class="preview-close" onclick={() => (previewWallpaper = null)} title="Close">
                <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                </svg>
            </button>
            <img class="preview-image" src={previewWallpaper.thumbs.large} alt={previewWallpaper.id} />
            {#if previewTags.length > 0}
                <div class="preview-tags">
                    {#each previewTags as tag (tag.id)}
                        <button class="tag-pill" onclick={() => searchTag(tag)}>{tag.name}</button>
                    {/each}
                </div>
            {:else if loadingTags}
                <div class="preview-tags-loading">Loading tags...</div>
            {/if}
            <div class="preview-actions">
                <span class="preview-resolution">{previewWallpaper.resolution}</span>
                <button class="preview-similar" onclick={() => searchSimilar(previewWallpaper!)} title="Find Similar">
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"/>
                    </svg>
                </button>
                <button class="preview-apply" onclick={() => { applyWallpaper(previewWallpaper!); previewWallpaper = null; }} title="Apply Wallpaper">
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                    </svg>
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    @keyframes pulse {
        0%, 100% { opacity: 0.5; }
        50% { opacity: 1; }
    }

    @keyframes shimmer {
        0% { background-position: -200% center; }
        100% { background-position: 200% center; }
    }

    :global(body) {
        overflow: hidden;
    }

    :root {
        margin: 0;
        padding: 0;
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        font-size: 12px;
        color: #f0f0f0;
        background-color: #161618;
    }

    /* Custom scrollbar */
    :global(::-webkit-scrollbar) {
        width: 6px;
    }
    :global(::-webkit-scrollbar-track) {
        background: transparent;
    }
    :global(::-webkit-scrollbar-thumb) {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 3px;
    }
    :global(::-webkit-scrollbar-thumb:hover) {
        background: rgba(255, 255, 255, 0.2);
    }

    .layout {
        display: flex;
        height: 100vh;
        width: 100%;
        background-color: #161618;
    }

    /* ─── Sidebar ─── */
    .sidebar {
        width: 46px;
        flex-shrink: 0;
        background: linear-gradient(180deg, #0e0e10 0%, #131316 100%);
        box-shadow: 2px 0 12px rgba(0, 0, 0, 0.4);
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 10px 0 20px;
        gap: 4px;
        overflow-y: auto;
        z-index: 2;
    }

    .nav-btn {
        width: 34px;
        height: 34px;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: #5a5a66;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        position: relative;
        transition:
            background 0.2s ease-out,
            color 0.2s ease-out,
            transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1),
            box-shadow 0.2s ease-out;
    }

    .nav-btn:hover {
        background: rgba(255, 255, 255, 0.06);
        color: #b0b0c0;
        transform: scale(1.08);
    }

    .nav-btn.active {
        background: rgba(100, 108, 255, 0.12);
        color: #8b8fff;
        box-shadow: 0 0 12px rgba(100, 108, 255, 0.15);
    }

    .nav-btn.active::before {
        content: '';
        position: absolute;
        left: -6px;
        top: 50%;
        transform: translateY(-50%);
        width: 3px;
        height: 18px;
        background: #646cff;
        border-radius: 0 3px 3px 0;
    }

    .nav-btn:active {
        transform: scale(0.95);
    }

    .divider {
        width: 22px;
        height: 1px;
        background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.08), transparent);
        margin: 6px 0;
    }

    .spacer {
        flex: 1;
    }

    /* ─── Main Content ─── */
    main {
        flex: 1;
        overflow-y: auto;
        min-width: 0;
        background: linear-gradient(180deg, #161618 0%, #1a1a1e 100%);
    }

    /* ─── Inputs (shared) ─── */
    .collection-picker {
        padding: 8px;
    }

    .collection-picker select {
        width: 100%;
        box-sizing: border-box;
        padding: 8px 12px;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.04);
        box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3);
        color: #e0e0e8;
        font-size: 12px;
        outline: none;
        transition: border-color 0.2s ease-out, box-shadow 0.2s ease-out;
    }

    .collection-picker select:focus {
        border-color: rgba(100, 108, 255, 0.5);
        box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3), 0 0 0 3px rgba(100, 108, 255, 0.1);
    }

    .collection-picker select option {
        background: #1e1e22;
        color: #e0e0e8;
    }

    .search-bar {
        padding: 8px;
    }

    .search-bar input {
        width: 100%;
        box-sizing: border-box;
        padding: 8px 12px;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.04);
        box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3);
        color: #e0e0e8;
        font-size: 12px;
        outline: none;
        transition: border-color 0.2s ease-out, box-shadow 0.2s ease-out;
    }

    .search-bar input:focus {
        border-color: rgba(100, 108, 255, 0.5);
        box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3), 0 0 0 3px rgba(100, 108, 255, 0.1);
    }

    .search-bar input::placeholder {
        color: #555;
    }

    /* ─── Status Messages ─── */
    .status {
        text-align: center;
        padding: 24px;
        color: #666;
        animation: pulse 1.8s ease-in-out infinite;
    }

    .error {
        color: #ff6b6b;
        animation: none;
    }

    /* ─── Wallpaper Grid ─── */
    .grid {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        gap: 6px;
        padding: 6px;
    }

    .thumb-wrapper {
        position: relative;
        overflow: hidden;
        border-radius: 8px;
        transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.25s ease-out;
    }

    .thumb-wrapper:hover {
        transform: scale(1.03);
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
        z-index: 1;
    }

    .thumb {
        width: 100%;
        cursor: pointer;
        border: none;
        padding: 0;
        background: none;
        display: block;
    }

    .thumb img {
        width: 100%;
        display: block;
        transition: filter 0.3s ease-out;
    }

    .thumb-wrapper:hover .thumb img {
        filter: brightness(1.1) saturate(1.15);
    }

    .thumb:hover:not(:disabled) {
        opacity: 1;
    }

    .thumb:disabled {
        cursor: default;
        opacity: 0.5;
    }

    .thumb-wrapper.setting .thumb {
        opacity: 1;
    }

    .applying {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(2px);
        color: #fff;
        font-size: 14px;
        font-weight: bold;
        animation: pulse 1.2s ease-in-out infinite;
    }

    .preview-icon {
        position: absolute;
        top: 6px;
        right: 6px;
        width: 26px;
        height: 26px;
        border: none;
        border-radius: 8px;
        background: rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(8px);
        color: rgba(255, 255, 255, 0.85);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        opacity: 0;
        transform: translateY(-4px);
        transition: opacity 0.2s ease-out, transform 0.2s ease-out, background 0.2s ease-out;
    }

    .preview-icon:hover {
        background: rgba(100, 108, 255, 0.6);
        color: #fff;
    }

    .thumb-wrapper:hover .preview-icon,
    .thumb-wrapper:hover .delete-icon {
        opacity: 1;
        transform: translateY(0);
    }

    .delete-icon {
        position: absolute;
        top: 6px;
        left: 6px;
        width: 26px;
        height: 26px;
        border: none;
        border-radius: 8px;
        background: rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(8px);
        color: #ff6b6b;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        opacity: 0;
        transform: translateY(-4px);
        transition: opacity 0.2s ease-out, transform 0.2s ease-out, background 0.2s ease-out;
    }

    .delete-icon:hover {
        background: rgba(255, 60, 60, 0.7);
        color: #fff;
    }

    /* ─── Preview Modal ─── */
    .preview-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.8);
        backdrop-filter: blur(16px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
        animation: fadeIn 0.2s ease-out;
    }

    .preview-content {
        position: relative;
        max-width: 90%;
        max-height: 90%;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        animation: fadeIn 0.25s ease-out;
    }

    .preview-close {
        position: absolute;
        top: -8px;
        right: -8px;
        width: 28px;
        height: 28px;
        border: none;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(8px);
        color: #fff;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        z-index: 1;
        transition: background 0.2s ease-out, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    .preview-close:hover {
        background: rgba(255, 255, 255, 0.25);
        transform: scale(1.1);
    }

    .preview-image {
        max-width: 538px;
        max-height: 70vh;
        border-radius: 10px;
        object-fit: contain;
        box-shadow: 0 8px 40px rgba(0, 0, 0, 0.5);
    }

    .preview-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 5px;
        max-width: 538px;
        justify-content: center;
    }

    .preview-tags-loading {
        color: #555;
        font-size: 11px;
        animation: pulse 1.8s ease-in-out infinite;
    }

    .tag-pill {
        padding: 4px 12px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 14px;
        background: rgba(255, 255, 255, 0.06);
        color: #aaa;
        font-size: 11px;
        cursor: pointer;
        transition:
            background 0.2s ease-out,
            border-color 0.2s ease-out,
            color 0.2s ease-out,
            transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    .tag-pill:hover {
        background: rgba(100, 108, 255, 0.15);
        border-color: rgba(100, 108, 255, 0.4);
        color: #c8caff;
        transform: scale(1.05);
    }

    .preview-actions {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .preview-resolution {
        color: #666;
        font-size: 12px;
        font-weight: 500;
    }

    .preview-similar {
        width: 36px;
        height: 36px;
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.04);
        color: #aaa;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        transition:
            background 0.2s ease-out,
            border-color 0.2s ease-out,
            transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1),
            box-shadow 0.2s ease-out;
    }

    .preview-similar:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(255, 255, 255, 0.25);
        transform: scale(1.1);
        box-shadow: 0 2px 12px rgba(0, 0, 0, 0.3);
    }

    .preview-apply {
        width: 36px;
        height: 36px;
        border: none;
        border-radius: 50%;
        background: linear-gradient(135deg, #646cff 0%, #535bf2 100%);
        color: #fff;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        transition:
            transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1),
            box-shadow 0.2s ease-out;
        box-shadow: 0 2px 12px rgba(100, 108, 255, 0.3);
    }

    .preview-apply:hover {
        transform: scale(1.1);
        box-shadow: 0 4px 20px rgba(100, 108, 255, 0.5);
    }

    .preview-apply:active {
        transform: scale(0.95);
    }

    /* ─── Settings Panel ─── */
    .settings-panel {
        padding: 18px;
        overflow-y: auto;
        height: 100%;
        box-sizing: border-box;
    }

    .settings-title {
        margin: 0 0 16px 0;
        font-size: 15px;
        font-weight: 600;
        color: #e0e0e8;
        letter-spacing: 0.3px;
    }

    .settings-form {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    .settings-label {
        display: flex;
        flex-direction: column;
        gap: 5px;
    }

    .settings-label-text {
        font-weight: 600;
        font-size: 10px;
        color: #666;
        text-transform: uppercase;
        letter-spacing: 0.8px;
    }

    .settings-field {
        display: flex;
        flex-direction: column;
        gap: 5px;
    }

    .settings-input {
        padding: 8px 12px;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        font-size: 12px;
        background: rgba(255, 255, 255, 0.04);
        box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3);
        color: #e0e0e8;
        outline: none;
        transition: border-color 0.2s ease-out, box-shadow 0.2s ease-out;
    }

    .settings-input:focus {
        border-color: rgba(100, 108, 255, 0.5);
        box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.3), 0 0 0 3px rgba(100, 108, 255, 0.1);
    }

    .settings-input::placeholder {
        color: #444;
    }

    .settings-input option {
        background: #1e1e22;
        color: #e0e0e8;
    }

    .settings-toggle-group {
        display: flex;
        gap: 6px;
    }

    .settings-toggle {
        flex: 1;
        padding: 7px 10px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 8px;
        font-size: 11px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s ease-out, transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
        color: #555;
        background: rgba(255, 255, 255, 0.03);
    }

    .settings-toggle:hover {
        background: rgba(255, 255, 255, 0.06);
        color: #888;
    }

    .settings-toggle:active {
        transform: scale(0.96);
    }

    .settings-toggle.sfw.active {
        background: linear-gradient(135deg, #3d6b35 0%, #2d5227 100%);
        color: #c8e6bf;
        border-color: rgba(90, 125, 79, 0.6);
        box-shadow: 0 0 10px rgba(74, 103, 65, 0.2);
    }

    .settings-toggle.sketchy.active {
        background: linear-gradient(135deg, #7a6a2a 0%, #5e5220 100%);
        color: #e6ddb3;
        border-color: rgba(158, 141, 66, 0.6);
        box-shadow: 0 0 10px rgba(138, 122, 58, 0.2);
    }

    .settings-toggle.nsfw.active {
        background: linear-gradient(135deg, #6b2a2a 0%, #521e1e 100%);
        color: #e6b3b3;
        border-color: rgba(158, 66, 66, 0.6);
        box-shadow: 0 0 10px rgba(122, 58, 58, 0.2);
    }

    .settings-toggle.category.active {
        background: linear-gradient(135deg, #2a4a6b 0%, #1e3852 100%);
        color: #b3d4e6;
        border-color: rgba(66, 120, 160, 0.6);
        box-shadow: 0 0 10px rgba(58, 90, 122, 0.2);
    }

    .settings-save {
        margin-top: 6px;
        padding: 9px 18px;
        border: none;
        border-radius: 8px;
        background: linear-gradient(135deg, #646cff 0%, #535bf2 100%);
        color: #fff;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.2s ease-out;
        box-shadow: 0 2px 12px rgba(100, 108, 255, 0.25);
    }

    .settings-save:hover {
        box-shadow: 0 4px 20px rgba(100, 108, 255, 0.4);
        background-size: 200% auto;
    }

    .settings-save:active {
        transform: scale(0.97);
    }

    .settings-danger {
        margin-top: 14px;
        padding-top: 14px;
        border-top: 1px solid rgba(255, 255, 255, 0.06);
    }

    .settings-clear-history {
        width: 100%;
        padding: 9px 18px;
        border: 1px solid rgba(158, 66, 66, 0.4);
        border-radius: 8px;
        background: rgba(122, 58, 58, 0.1);
        color: #d4a0a0;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.2s ease-out, border-color 0.2s ease-out, box-shadow 0.2s ease-out;
    }

    .settings-clear-history:hover {
        background: rgba(122, 58, 58, 0.25);
        border-color: rgba(158, 66, 66, 0.6);
        box-shadow: 0 0 12px rgba(158, 66, 66, 0.15);
    }

    /* ─── Queue View ─── */
    .queue-icon {
        position: absolute;
        bottom: 6px;
        left: 6px;
        width: 26px;
        height: 26px;
        border: none;
        border-radius: 8px;
        background: rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(8px);
        color: rgba(255, 255, 255, 0.85);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        opacity: 0;
        transform: translateY(4px);
        transition: opacity 0.2s ease-out, transform 0.2s ease-out, background 0.2s ease-out;
    }

    .queue-icon:hover {
        background: rgba(80, 200, 120, 0.6);
        color: #fff;
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

    .queue-view {
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .queue-controls {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 10px 6px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
        flex-shrink: 0;
    }

    .queue-play-btn {
        display: flex;
        align-items: center;
        gap: 5px;
        padding: 6px 12px;
        border: none;
        border-radius: 8px;
        background: linear-gradient(135deg, #646cff 0%, #535bf2 100%);
        color: #fff;
        font-size: 11px;
        font-weight: 600;
        cursor: pointer;
        transition: box-shadow 0.2s ease-out, transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
        white-space: nowrap;
        flex-shrink: 0;
        box-shadow: 0 2px 10px rgba(100, 108, 255, 0.25);
    }

    .queue-play-btn.running {
        background: linear-gradient(135deg, #ff6b6b 0%, #ee4444 100%);
        box-shadow: 0 2px 10px rgba(255, 80, 80, 0.25);
    }

    .queue-play-btn:disabled {
        opacity: 0.4;
        cursor: default;
    }

    .queue-play-btn:not(:disabled):hover {
        box-shadow: 0 4px 16px rgba(100, 108, 255, 0.4);
        transform: scale(1.03);
    }

    .queue-play-btn.running:not(:disabled):hover {
        box-shadow: 0 4px 16px rgba(255, 80, 80, 0.4);
    }

    .queue-interval {
        display: flex;
        align-items: center;
        gap: 6px;
        flex: 1;
        min-width: 0;
    }

    .queue-interval-label {
        font-size: 10px;
        color: rgba(255, 255, 255, 0.4);
        white-space: nowrap;
        flex-shrink: 0;
    }

    .queue-interval-pills {
        display: flex;
        gap: 3px;
        flex-wrap: wrap;
    }

    .queue-interval-pill {
        padding: 3px 7px;
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 6px;
        background: rgba(255, 255, 255, 0.05);
        color: rgba(255, 255, 255, 0.5);
        font-size: 10px;
        cursor: pointer;
        transition: background 0.15s ease-out, color 0.15s ease-out, border-color 0.15s ease-out;
    }

    .queue-interval-pill:hover {
        background: rgba(255, 255, 255, 0.1);
        color: rgba(255, 255, 255, 0.8);
    }

    .queue-interval-pill.active {
        background: rgba(100, 108, 255, 0.25);
        border-color: rgba(100, 108, 255, 0.5);
        color: #b0b4ff;
    }

    .thumb-wrapper.queue-active {
        box-shadow: 0 0 0 2px rgba(100, 108, 255, 0.8), 0 0 16px rgba(100, 108, 255, 0.4);
    }

    /* Item being dragged — looks lifted/ghosted */
    .thumb-wrapper.drag-source {
        opacity: 0.35;
        transform: scale(0.93);
        box-shadow: 0 0 0 2px rgba(100, 108, 255, 0.5), 0 8px 24px rgba(0, 0, 0, 0.6);
        z-index: 0;
    }

    /* Drop target — bright accent ring + scale */
    .thumb-wrapper.drag-over {
        box-shadow: 0 0 0 2px #646cff, 0 0 20px rgba(100, 108, 255, 0.55);
        transform: scale(1.06);
        z-index: 2;
    }

    /* All other items dim during drag */
    .grid.dragging .thumb-wrapper:not(.drag-over):not(.drag-source) {
        opacity: 0.55;
        transform: none;
        box-shadow: none;
    }

    .grid.dragging,
    .grid.dragging .thumb-wrapper {
        cursor: grabbing;
    }

    .drag-handle {
        position: absolute;
        top: 6px;
        right: 6px;
        width: 22px;
        height: 22px;
        border-radius: 6px;
        background: rgba(0, 0, 0, 0.45);
        backdrop-filter: blur(8px);
        color: rgba(255, 255, 255, 0.6);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: grab;
        opacity: 0;
        transform: translateY(-4px);
        transition: opacity 0.2s ease-out, transform 0.2s ease-out, background 0.15s ease-out;
    }

    .drag-handle:hover {
        background: rgba(100, 108, 255, 0.5);
        color: #fff;
    }

    .thumb-wrapper:hover .drag-handle,
    .grid.dragging .drag-handle {
        opacity: 1;
        transform: translateY(0);
    }

    .grid.dragging .drag-source .drag-handle {
        background: rgba(100, 108, 255, 0.7);
        color: #fff;
        cursor: grabbing;
    }

    .queue-footer {
        padding: 8px 10px;
        display: flex;
        justify-content: center;
        flex-shrink: 0;
    }

    .queue-clear-btn {
        padding: 6px 14px;
        border: 1px solid rgba(158, 66, 66, 0.4);
        border-radius: 8px;
        background: rgba(122, 58, 58, 0.1);
        color: #d4a0a0;
        font-size: 11px;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.2s ease-out, border-color 0.2s ease-out;
    }

    .queue-clear-btn:hover {
        background: rgba(122, 58, 58, 0.25);
        border-color: rgba(158, 66, 66, 0.6);
    }

    .grid.dragging .thumb-wrapper {
        cursor: grabbing;
    }
</style>
