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

</script>

<div class="flex h-screen w-full bg-base-100 overflow-hidden">
    <!-- ─── Sidebar ─── -->
    <nav class="w-[46px] flex-shrink-0 flex flex-col items-center py-2.5 pb-5 gap-1 bg-base-200 shadow-[2px_0_12px_rgba(0,0,0,0.4)] z-10 overflow-y-auto">
        <button class="nav-btn" class:active={activeView.kind === "query"} title="Search" onclick={activateSearch}>
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
            </svg>
        </button>
        <button class="nav-btn" class:active={activeView.kind === "history"} title="History" onclick={loadHistory}>
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm0 18c-4.4 0-8-3.6-8-8s3.6-8 8-8 8 3.6 8 8-3.6 8-8 8zm.5-13H11v6l5.2 3.1.8-1.3-4.5-2.7V7z"/>
            </svg>
        </button>
        <button class="nav-btn" class:active={activeView.kind === "queue"} title="Queue" onclick={activateQueue}>
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z"/>
            </svg>
        </button>

        <div class="w-[22px] h-px bg-gradient-to-r from-transparent via-base-content/10 to-transparent my-1"></div>

        {#each categories as cat}
            <button class="nav-btn" class:active={isActive(cat)} title={cat.label} onclick={() => loadSearch(cat.sorting)}>
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
                title="Collections"
                onclick={activateCollections}
            >
                <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                    <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                </svg>
            </button>
        {/if}

        <div class="flex-1"></div>

        <button class="nav-btn" title="Undo" onclick={undoWallpaper} disabled={undoing}>
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M12.5 8c-2.65 0-5.05.99-6.9 2.6L2 7v9h9l-3.62-3.62c1.39-1.16 3.16-1.88 5.12-1.88 3.54 0 6.55 2.31 7.6 5.5l2.37-.78C21.08 11.03 17.15 8 12.5 8z"/>
            </svg>
        </button>
        <button class="nav-btn" class:active={activeView.kind === "settings"} title="Settings" onclick={openSettings}>
            <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.49.49 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.49.49 0 00-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6A3.6 3.6 0 1112 8.4a3.6 3.6 0 010 7.2z"/>
            </svg>
        </button>
    </nav>

    <!-- ─── Main Content ─── -->
    <main bind:this={mainEl} onscroll={onScroll} class="flex-1 overflow-y-auto min-w-0 bg-gradient-to-b from-base-100 to-base-200/50">

        <!-- Collections picker -->
        {#if activeView.kind === "collections" || activeView.kind === "collection"}
            <div class="p-2">
                <select
                    class="select select-sm select-bordered w-full bg-base-200 border-base-300"
                    value={selectedCollectionId ?? ""}
                    onchange={(e) => {
                        const val = (e.target as HTMLSelectElement).value;
                        if (val) selectCollection(Number(val));
                    }}
                >
                    <option value="" disabled>Select a collection...</option>
                    {#each collections as col (col.id)}
                        <option value={col.id}>{col.label} ({col.count})</option>
                    {/each}
                </select>
            </div>
        {/if}

        <!-- Search bar -->
        {#if activeView.kind === "query"}
            <form class="p-2" onsubmit={(e) => { e.preventDefault(); searchQuery(); }}>
                <input
                    type="text"
                    class="input input-sm input-bordered w-full bg-base-200 border-base-300 placeholder:text-base-content/25 focus:outline-none focus:border-primary"
                    bind:value={searchInput}
                    placeholder="Search wallpapers..."
                />
            </form>
        {/if}

        {#if activeView.kind === "settings"}
            <!-- ─── Settings ─── -->
            <div class="p-[18px] h-full box-border overflow-y-auto">
                <h3 class="mt-0 mb-4 text-[15px] font-semibold text-base-content tracking-[0.3px]">Settings</h3>
                <form class="flex flex-col gap-3.5" onsubmit={(e) => { e.preventDefault(); saveSettings(); }}>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Username</span>
                        <input type="text" class="input input-sm input-bordered bg-base-200 border-base-300 placeholder:text-base-content/20" bind:value={settingsUsername} placeholder="wallhaven username" />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">API Key</span>
                        <input type="password" class="input input-sm input-bordered bg-base-200 border-base-300 placeholder:text-base-content/20" bind:value={settingsApiKey} placeholder="wallhaven API key" />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Categories</span>
                        <div class="join w-full">
                            <button type="button"
                                class="join-item btn btn-sm flex-1 border-base-300 {settingsGeneral ? 'bg-info/20 text-info border-info/40' : 'bg-base-200 text-base-content/30'}"
                                onclick={() => (settingsGeneral = !settingsGeneral)}>General</button>
                            <button type="button"
                                class="join-item btn btn-sm flex-1 border-base-300 {settingsAnime ? 'bg-info/20 text-info border-info/40' : 'bg-base-200 text-base-content/30'}"
                                onclick={() => (settingsAnime = !settingsAnime)}>Anime</button>
                            <button type="button"
                                class="join-item btn btn-sm flex-1 border-base-300 {settingsPeople ? 'bg-info/20 text-info border-info/40' : 'bg-base-200 text-base-content/30'}"
                                onclick={() => (settingsPeople = !settingsPeople)}>People</button>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Purity</span>
                        <div class="join w-full">
                            <button type="button"
                                class="join-item btn btn-sm flex-1 border-base-300 {settingsSfw ? 'bg-success/20 text-success border-success/40' : 'bg-base-200 text-base-content/30'}"
                                onclick={() => (settingsSfw = !settingsSfw)}>SFW</button>
                            <button type="button"
                                class="join-item btn btn-sm flex-1 border-base-300 {settingsSketchy ? 'bg-warning/20 text-warning border-warning/40' : 'bg-base-200 text-base-content/30'}"
                                onclick={() => (settingsSketchy = !settingsSketchy)}>Sketchy</button>
                            <button type="button"
                                class="join-item btn btn-sm flex-1 border-base-300 {settingsNsfw ? 'bg-error/20 text-error border-error/40' : 'bg-base-200 text-base-content/30'}"
                                onclick={() => (settingsNsfw = !settingsNsfw)}>NSFW</button>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Minimum Resolution</span>
                        <select class="select select-sm select-bordered bg-base-200 border-base-300" bind:value={settingsAtleast}>
                            <option value="">Any</option>
                            {#each availableResolutions as res}
                                <option value={res}>{res}</option>
                            {/each}
                        </select>
                    </div>
                    <button class="btn btn-primary btn-sm mt-1" type="submit">Save</button>
                </form>
                <div class="mt-3.5 pt-3.5 border-t border-base-300">
                    <button class="btn btn-error btn-outline btn-sm w-full" onclick={clearHistory}>Clear History</button>
                </div>
            </div>

        {:else if activeView.kind === "queue"}
            <!-- ─── Queue ─── -->
            <div class="flex flex-col h-full">
                <div class="flex items-center gap-2.5 px-2.5 py-2 border-b border-base-300 flex-shrink-0">
                    <button
                        class="btn btn-sm flex-shrink-0 gap-1.5 {queueRunning ? 'btn-error' : 'btn-primary'}"
                        onclick={() => queueRunning ? stopCycling() : startCycling()}
                        disabled={queue.length === 0}
                        title={queueRunning ? "Stop cycling" : "Start cycling"}
                    >
                        {#if queueRunning}
                            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
                            Stop
                        {:else}
                            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
                            Start
                        {/if}
                    </button>
                    <div class="flex items-center gap-1.5">
                        <span class="text-[10px] text-base-content/40 whitespace-nowrap">Every</span>
                        <input
                            type="number"
                            class="input input-xs input-bordered w-14 bg-base-200 border-base-300 text-center [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
                            min="1"
                            max="1440"
                            bind:value={queueIntervalMinutes}
                            onchange={() => { if (queueIntervalMinutes > 0) changeQueueInterval(queueIntervalMinutes); }}
                        />
                        <span class="text-[10px] text-base-content/40 whitespace-nowrap">min</span>
                    </div>
                </div>

                {#if queue.length === 0}
                    <p class="text-center py-6 text-base-content/40 text-xs animate-pulse">Queue is empty — add wallpapers from search views</p>
                {:else}
                    <div class="grid grid-cols-3 gap-1.5 p-1.5" class:dragging={queueDragging}>
                        {#each queue as wp, idx (wp.id)}
                            <div
                                class="thumb-wrapper group relative overflow-hidden rounded-lg transition-all duration-[250ms] ease-[cubic-bezier(0.34,1.56,0.64,1)]"
                                class:setting={settingWallpaper === wp.id}
                                class:queue-active={idx === queueIndex % queue.length && queueRunning}
                                class:drag-source={queueDragging && dragFromIndex === idx}
                                class:drag-over={dragOverIndex === idx && queueDragging && dragFromIndex !== idx}
                                data-queue-idx={idx}
                            >
                                <button
                                    class="thumb w-full border-0 p-0 bg-transparent block cursor-pointer disabled:cursor-default disabled:opacity-50"
                                    onclick={() => applyWallpaper(wp)}
                                    disabled={settingWallpaper !== ""}
                                >
                                    <img src={wp.thumbs.small} alt={wp.id} class="w-full block" />
                                    {#if settingWallpaper === wp.id}
                                        <span class="absolute inset-0 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                                            <span class="loading loading-spinner loading-sm text-white"></span>
                                        </span>
                                    {/if}
                                </button>
                                <button
                                    class="absolute top-1.5 left-1.5 w-[26px] h-[26px] flex items-center justify-center rounded-lg p-0 cursor-pointer border-0 bg-black/50 backdrop-blur-[8px] text-error opacity-0 -translate-y-1 group-hover:opacity-100 group-hover:translate-y-0 hover:bg-error/70 hover:text-white transition-all duration-200"
                                    onclick={(e) => { e.stopPropagation(); removeFromQueue(wp); }}
                                    title="Remove from queue"
                                >
                                    <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
                                </button>
                                <!-- svelte-ignore a11y_no_static_element_interactions -->
                                <div
                                    class="drag-handle absolute top-1.5 right-1.5 w-[22px] h-[22px] flex items-center justify-center rounded-md bg-black/45 backdrop-blur-[8px] text-white/60 cursor-grab opacity-0 -translate-y-1 group-hover:opacity-100 group-hover:translate-y-0 hover:bg-primary/50 hover:text-white transition-all duration-200"
                                    title="Drag to reorder"
                                    onmousedown={(e) => onDragHandleMouseDown(e, idx)}
                                >
                                    <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/></svg>
                                </div>
                            </div>
                        {/each}
                    </div>
                    <div class="flex justify-center px-2.5 py-2 flex-shrink-0">
                        <button class="btn btn-error btn-outline btn-xs" onclick={clearQueue}>Clear Queue</button>
                    </div>
                {/if}
            </div>

        {:else if loading}
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
                            onclick={() => applyWallpaper(wp)}
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
                            onclick={() => openPreview(wp)}
                            title="Preview"
                        >
                            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/></svg>
                        </button>

                        <!-- Queue icon (bottom-left) -->
                        <button
                            class="queue-icon absolute bottom-1.5 left-1.5 w-[26px] h-[26px] flex items-center justify-center rounded-lg p-0 cursor-pointer border-0 bg-black/50 backdrop-blur-[8px] text-white/85 transition-all duration-200"
                            class:in-queue={queue.some((q) => q.id === wp.id)}
                            onclick={(e) => { e.stopPropagation(); toggleQueue(wp); }}
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
                                onclick={(e) => { e.stopPropagation(); deleteHistoryEntry(wp); }}
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
    </main>
</div>

<!-- ─── Preview Modal ─── -->
{#if previewWallpaper}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="fixed inset-0 bg-black/80 backdrop-blur-[16px] flex items-center justify-center z-[100] animate-[fadeIn_0.2s_ease-out]"
        onclick={() => (previewWallpaper = null)}
        onkeydown={(e) => { if (e.key === 'Escape') previewWallpaper = null; }}
    >
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
            class="relative max-w-[90%] max-h-[90%] flex flex-col items-center gap-3 animate-[fadeIn_0.25s_ease-out]"
            onclick={(e) => e.stopPropagation()}
        >
            <button
                class="absolute -top-2 -right-2 w-7 h-7 flex items-center justify-center rounded-full border-0 p-0 cursor-pointer z-10 bg-white/10 backdrop-blur-[8px] text-white hover:bg-white/25 hover:scale-110 active:scale-95 transition-all duration-200"
                onclick={() => (previewWallpaper = null)}
                title="Close"
            >
                <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
            </button>

            <img
                class="max-w-[480px] max-h-[70vh] rounded-xl object-contain shadow-[0_8px_40px_rgba(0,0,0,0.5)]"
                src={previewWallpaper.thumbs.large}
                alt={previewWallpaper.id}
            />

            {#if loadingTags}
                <p class="text-[11px] text-base-content/30 animate-pulse">Loading tags...</p>
            {:else if previewTags.length > 0}
                <div class="flex flex-wrap gap-1 max-w-[480px] justify-center">
                    {#each previewTags as tag (tag.id)}
                        <button
                            class="px-3 py-1 rounded-full border border-white/10 bg-white/[0.06] text-base-content/55 text-[11px] cursor-pointer transition-all duration-200 hover:bg-primary/15 hover:border-primary/40 hover:text-primary-content hover:scale-105"
                            onclick={() => searchTag(tag)}
                        >{tag.name}</button>
                    {/each}
                </div>
            {/if}

            <div class="flex items-center gap-2.5">
                <span class="text-base-content/40 text-xs font-medium">{previewWallpaper.resolution}</span>
                <button
                    class="w-9 h-9 flex items-center justify-center rounded-full border border-white/[0.12] bg-white/[0.04] text-base-content/55 cursor-pointer p-0 hover:bg-white/10 hover:border-white/25 hover:scale-110 active:scale-95 transition-all duration-200"
                    onclick={() => searchSimilar(previewWallpaper!)}
                    title="Find Similar"
                >
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"/></svg>
                </button>
                <button
                    class="w-9 h-9 flex items-center justify-center rounded-full border-0 bg-primary text-primary-content cursor-pointer p-0 shadow-[0_2px_12px_oklch(var(--p)/0.3)] hover:scale-110 hover:shadow-[0_4px_20px_oklch(var(--p)/0.5)] active:scale-95 transition-all duration-200"
                    onclick={() => { applyWallpaper(previewWallpaper!); previewWallpaper = null; }}
                    title="Apply Wallpaper"
                >
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
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

    :global(body) {
        overflow: hidden;
    }

    /* Nav button — needs position:relative for ::before and custom transitions */
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

    /* Left accent bar on active nav item */
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

    /* Thumb hover effects — group-hover in Tailwind handles siblings,
       but the filter on img and scale on wrapper need CSS for specificity */
    .thumb-wrapper:hover {
        transform: scale(1.03);
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
        z-index: 1;
    }

    .thumb-wrapper:hover .thumb img {
        filter: brightness(1.1) saturate(1.15);
    }

    /* Queue icon — hidden by default, visible on hover and when in-queue */
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

    /* Queue drag states */
    .thumb-wrapper.queue-active {
        box-shadow: 0 0 0 2px oklch(var(--p) / 0.8), 0 0 16px oklch(var(--p) / 0.4);
    }

    .thumb-wrapper.drag-source {
        opacity: 0.35;
        transform: scale(0.93);
        box-shadow: 0 0 0 2px oklch(var(--p) / 0.5), 0 8px 24px rgba(0, 0, 0, 0.6);
        z-index: 0;
    }

    .thumb-wrapper.drag-over {
        box-shadow: 0 0 0 2px oklch(var(--p)), 0 0 20px oklch(var(--p) / 0.55);
        transform: scale(1.06);
        z-index: 2;
    }

    .grid.dragging .thumb-wrapper:not(.drag-over):not(.drag-source) {
        opacity: 0.55;
        transform: none;
        box-shadow: none;
    }

    .grid.dragging,
    .grid.dragging .thumb-wrapper {
        cursor: grabbing;
    }

    /* Show all drag handles while dragging, style active source handle */
    .grid.dragging .drag-handle {
        opacity: 1;
        transform: translateY(0);
    }

    .grid.dragging .thumb-wrapper.drag-source .drag-handle {
        background: oklch(var(--p) / 0.7);
        color: #fff;
        cursor: grabbing;
    }
</style>
