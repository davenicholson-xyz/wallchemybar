<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, tick } from "svelte";
    import type { Tag, Wallpaper, Collection, View } from "$lib/types";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import WallpaperGrid from "$lib/components/WallpaperGrid.svelte";
    import PreviewModal from "$lib/components/PreviewModal.svelte";
    import QueuePanel from "$lib/components/QueuePanel.svelte";
    import SettingsPanel from "$lib/components/SettingsPanel.svelte";
    import CollectionCycleBar from "$lib/components/CollectionCycleBar.svelte";

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

    // ─── Shared state ────────────────────────────────────────────────────────────
    let collections: Collection[] = $state([]);
    let activeView = $state<View>({ kind: "search", sorting: "hot" });
    let wallpapers: Wallpaper[] = $state([]);
    let loading = $state(true);
    let loadingMore = $state(false);
    let error = $state("");
    let settingWallpaper = $state("");
    let page = $state(1);
    let hasMore = $state(true);
    let mainEl: HTMLElement;
    let searchInput = $state("");
    let searchInputEl: HTMLInputElement | undefined;
    let selectedCollectionId: number | null = $state(null);
    let searchSeed: string | null = null;
    let selectedIndex = $state(-1);

    let isGridView = $derived(
        activeView.kind !== "settings" && activeView.kind !== "queue"
    );

    $effect(() => {
        if (loading) selectedIndex = -1;
    });
    let undoing = $state(false);

    // ─── Preview state ───────────────────────────────────────────────────────────
    let previewWallpaper: Wallpaper | null = $state(null);
    let previewTags: Tag[] = $state([]);
    let loadingTags = $state(false);

    // ─── Queue state (persists across view changes) ───────────────────────────────
    let queue: Wallpaper[] = $state([]);
    let queueIntervalMinutes = $state(30);
    let queueRunning = $state(false);
    let queueIndex = $state(0);
    let queueTimerId: ReturnType<typeof setInterval> | undefined;

    // ─── Collection cycling state ─────────────────────────────────────────────
    let collectionCycleRunning = $state(false);
    let collectionCycleIntervalMinutes = $state(30);
    let collectionCycleTimerId: ReturnType<typeof setInterval> | undefined;
    let collectionCyclePage = $state(1);
    let collectionCyclePageIndex = $state(0);
    let collectionCycleBuffer: Wallpaper[] = [];

    // ─── Lifecycle ───────────────────────────────────────────────────────────────
    onMount(async () => {
        invoke("fetch_collections")
            .then((cols) => { collections = cols as Collection[]; })
            .catch(() => {});
        invoke("load_settings").then((s: any) => {
            if (s.collection_cycle_interval_minutes) {
                collectionCycleIntervalMinutes = s.collection_cycle_interval_minutes;
            }
            // Restore last-used collection ID so CollectionCycleBar is pre-configured,
            // but do NOT call loadCollection here — it would race with loadSearch("hot").
            if (s.collection_cycle_collection_id) {
                selectedCollectionId = s.collection_cycle_collection_id;
            }
        }).catch(() => {});
        await loadSearch("hot");
    });

    // ─── Search / browse ─────────────────────────────────────────────────────────
    async function loadSearch(sorting: string) {
        console.log(`[wallchemybar] loadSearch: sorting=${sorting}`);
        activeView = { kind: "search", sorting };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        searchSeed = sorting === "random" ? String(Math.floor(Math.random() * 1e9)) : null;
        try {
            const results: Wallpaper[] = await invoke("fetch_search", { sorting, page: 1, seed: searchSeed });
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
            const results: Wallpaper[] = await invoke("fetch_collection_wallpapers", { collectionId: id, page: 1 });
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
        if (collectionCycleRunning) stopCollectionCycling();
        selectedCollectionId = id;
        collectionCyclePage = 1; collectionCyclePageIndex = 0; collectionCycleBuffer = [];
        saveCollectionCyclePrefs();
        await loadCollection(id);
    }

    async function activateSearch() {
        activeView = { kind: "query", query: "" };
        wallpapers = [];
        loading = false;
        error = "";
        await tick();
        searchInputEl?.focus();
    }

    async function searchQuery() {
        const q = searchInput.trim();
        if (!q) return;
        activeView = { kind: "query", query: q };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        searchSeed = String(Math.floor(Math.random() * 1e9));
        try {
            const results: Wallpaper[] = await invoke("fetch_search", { sorting: "random", page: 1, query: q, seed: searchSeed });
            wallpapers = results;
            hasMore = results.length >= 24;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
        if (wallpapers.length > 0) {
            await tick();
            selectedIndex = 0;
            searchInputEl?.blur();
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
                results = await invoke("fetch_search", { sorting: activeView.sorting, page: nextPage, seed: searchSeed });
            } else if (activeView.kind === "query") {
                results = await invoke("fetch_search", { sorting: "random", page: nextPage, query: activeView.query, seed: searchSeed });
            } else if (activeView.kind === "collection") {
                results = await invoke("fetch_collection_wallpapers", { collectionId: activeView.id, page: nextPage });
            } else {
                console.warn(`[wallchemybar] loadNextPage: unexpected view kind: ${activeView.kind}`);
                loadingMore = false;
                return;
            }
            const elapsed = (performance.now() - startTime).toFixed(0);
            const existingIds = new Set(wallpapers.map((w) => w.id));
            const newResults = results.filter((w) => !existingIds.has(w.id));
            const dupes = results.length - newResults.length;
            if (dupes > 0) console.warn(`[wallchemybar] loadNextPage: filtered ${dupes} duplicate wallpapers from page ${nextPage}`);
            console.log(`[wallchemybar] loadNextPage: got ${results.length} results (${newResults.length} new) for page ${nextPage} in ${elapsed}ms`);
            wallpapers = [...wallpapers, ...newResults];
            page = nextPage;
            hasMore = results.length >= 24;
            console.log(`[wallchemybar] loadNextPage: hasMore=${hasMore}, total wallpapers=${wallpapers.length}`);
        } catch (e) {
            console.error(`[wallchemybar] loadNextPage: error on page ${nextPage}:`, e);
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

    // ─── Wallpaper actions ───────────────────────────────────────────────────────
    async function applyWallpaper(wp: Wallpaper) {
        invoke("hide_main");
        settingWallpaper = wp.id;
        const start = Date.now();
        try {
            await invoke("set_wallpaper", { wallpaper: wp });
        } catch (e) {
            error = String(e);
        } finally {
            const remaining = 600 - (Date.now() - start);
            if (remaining > 0) await new Promise(r => setTimeout(r, remaining));
            settingWallpaper = "";
        }
    }

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

    // ─── History ─────────────────────────────────────────────────────────────────
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

    // ─── Preview ─────────────────────────────────────────────────────────────────
    async function openPreview(wp: Wallpaper) {
        previewWallpaper = wp;
        previewTags = [];
        loadingTags = true;
        try {
            const tags: Tag[] = await invoke("fetch_wallpaper_tags", { wallpaperId: wp.id });
            if (previewWallpaper?.id === wp.id) previewTags = tags;
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
            const results: Wallpaper[] = await invoke("fetch_search", { sorting: "relevance", page: 1, query: q });
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
            const results: Wallpaper[] = await invoke("fetch_search", { sorting: "relevance", page: 1, query: q });
            wallpapers = results;
            hasMore = results.length >= 24;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    function isActive(cat: { sorting: string }): boolean {
        return activeView.kind === "search" && activeView.sorting === cat.sorting;
    }

    // ─── Queue ───────────────────────────────────────────────────────────────────
    async function activateQueue() {
        activeView = { kind: "queue" };
        try {
            queue = await invoke("get_queue");
        } catch (e) {
            error = String(e);
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
        if (collectionCycleRunning) stopCollectionCycling();
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
        // queueIntervalMinutes already updated by $bindable; just restart timer if needed
        if (queueRunning) {
            clearInterval(queueTimerId);
            queueTimerId = setInterval(advanceQueue, minutes * 60 * 1000);
        }
    }

    // ─── Collection cycling ───────────────────────────────────────────────────────
    function startCollectionCycling() {
        if (!selectedCollectionId) return;
        if (queueRunning) stopCycling();
        collectionCycleRunning = true;
        collectionCyclePage = 1;
        collectionCyclePageIndex = 0;
        collectionCycleBuffer = [];
        collectionCycleTimerId = setInterval(advanceCollectionCycle, collectionCycleIntervalMinutes * 60 * 1000);
    }

    function stopCollectionCycling() {
        collectionCycleRunning = false;
        clearInterval(collectionCycleTimerId);
        collectionCycleTimerId = undefined;
    }

    async function advanceCollectionCycle() {
        if (!selectedCollectionId) { stopCollectionCycling(); return; }
        if (collectionCyclePageIndex >= collectionCycleBuffer.length) {
            try {
                const results: Wallpaper[] = await invoke("fetch_collection_wallpapers", {
                    collectionId: selectedCollectionId, page: collectionCyclePage
                });
                if (results.length === 0) { stopCollectionCycling(); return; }
                collectionCycleBuffer = results;
                collectionCyclePageIndex = 0;
                collectionCyclePage = results.length < 24 ? 1 : collectionCyclePage + 1;
            } catch { return; }
        }
        await applyWallpaper(collectionCycleBuffer[collectionCyclePageIndex++]);
    }

    function changeCollectionCycleInterval(minutes: number) {
        collectionCycleIntervalMinutes = minutes;
        if (collectionCycleRunning) {
            clearInterval(collectionCycleTimerId);
            collectionCycleTimerId = setInterval(advanceCollectionCycle, minutes * 60 * 1000);
        }
        saveCollectionCyclePrefs();
    }

    function saveCollectionCyclePrefs() {
        invoke("load_settings").then((s: any) => {
            invoke("save_settings", { settings: {
                ...s,
                collection_cycle_collection_id: selectedCollectionId ?? 0,
                collection_cycle_interval_minutes: collectionCycleIntervalMinutes,
            }}).catch(() => {});
        });
    }

    async function removeFromQueue(wp: Wallpaper) {
        try {
            await invoke("remove_from_queue", { wallpaperId: wp.id });
            queue = await invoke("get_queue");
            if (queueIndex >= queue.length && queueIndex > 0) queueIndex = queue.length - 1;
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

    async function reorderQueue(reordered: Wallpaper[]) {
        queue = reordered;
        invoke("reorder_queue", { wallpapers: reordered }).catch(err => { error = String(err); });
    }

    function openSettings() {
        activeView = { kind: "settings" };
    }

    function handleKeyDown(e: KeyboardEvent) {
        const target = e.target as HTMLElement;
        const inInput = target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT";

        if (e.key === "Escape" && previewWallpaper) {
            previewWallpaper = null;
            return;
        }
        if (previewWallpaper) return;

        if (!inInput) {
            switch (e.key) {
                case "S": e.preventDefault(); activateSearch(); return;
                case "N": e.preventDefault(); loadSearch("date_added"); return;
                case "O": e.preventDefault(); loadSearch("hot"); return;
                case "T": e.preventDefault(); loadSearch("toplist"); return;
                case "R": e.preventDefault(); loadSearch("random"); return;
                case "C": e.preventDefault(); activateCollections(); return;
                case "H": e.preventDefault(); loadHistory(); return;
                case "Q": e.preventDefault(); activateQueue(); return;
                case "U": e.preventDefault(); undoWallpaper(); return;
                case ",": e.preventDefault(); openSettings(); return;
                case "E": e.preventDefault(); invoke("open_expanded"); return;
            }
        }

        if (!isGridView || inInput || wallpapers.length === 0) return;

        switch (e.key) {
            case "ArrowRight":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.min(selectedIndex + 1, wallpapers.length - 1);
                break;
            case "ArrowLeft":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.max(selectedIndex - 1, 0);
                break;
            case "ArrowDown":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.min(selectedIndex + 3, wallpapers.length - 1);
                break;
            case "ArrowUp":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.max(selectedIndex - 3, 0);
                break;
            case "h":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.max(selectedIndex - 1, 0);
                break;
            case "l":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.min(selectedIndex + 1, wallpapers.length - 1);
                break;
            case "j":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.min(selectedIndex + 3, wallpapers.length - 1);
                break;
            case "k":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.max(selectedIndex - 3, 0);
                break;
            case " ":
                if (selectedIndex >= 0 && wallpapers[selectedIndex]) {
                    e.preventDefault();
                    openPreview(wallpapers[selectedIndex]);
                }
                break;
            case "a":
                if (selectedIndex >= 0 && wallpapers[selectedIndex]) {
                    e.preventDefault();
                    toggleQueue(wallpapers[selectedIndex]);
                }
                break;
            case "Enter":
                if (selectedIndex >= 0 && wallpapers[selectedIndex]) {
                    e.preventDefault();
                    applyWallpaper(wallpapers[selectedIndex]);
                }
                break;
        }
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="flex h-screen w-full bg-base-100 overflow-hidden">
    <Sidebar
        {activeView}
        {collections}
        {undoing}
        {categories}
        isactive={isActive}
        onloadsearch={loadSearch}
        onloadhistory={loadHistory}
        onactivatequeue={activateQueue}
        onactivatesearch={activateSearch}
        onactivatecollections={activateCollections}
        onopensettings={openSettings}
        onundowallpaper={undoWallpaper}
        onexpand={() => invoke("open_expanded")}
    />

    <main bind:this={mainEl} onscroll={onScroll} class="flex-1 overflow-y-auto min-w-0 bg-gradient-to-b from-base-100 to-base-200/50">

        <!-- Collections picker -->
        {#if activeView.kind === "collections" || activeView.kind === "collection"}
            <div class="p-2 flex flex-col gap-1.5">
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
                {#if selectedCollectionId !== null}
                    <CollectionCycleBar
                        cycling={collectionCycleRunning}
                        bind:intervalMinutes={collectionCycleIntervalMinutes}
                        onstart={startCollectionCycling}
                        onstop={stopCollectionCycling}
                        onchangeinterval={changeCollectionCycleInterval}
                    />
                {/if}
            </div>
        {/if}

        <!-- Search bar -->
        {#if activeView.kind === "query"}
            <form class="p-2" onsubmit={(e) => { e.preventDefault(); searchQuery(); }}>
                <input
                    type="text"
                    class="input input-sm input-bordered w-full bg-base-200 border-base-300 placeholder:text-base-content/25 focus:outline-none focus:border-primary"
                    bind:this={searchInputEl}
                    bind:value={searchInput}
                    placeholder="Search wallpapers..."
                />
            </form>
        {/if}

        {#if activeView.kind === "settings"}
            <SettingsPanel onreloadsearch={loadSearch} />

        {:else if activeView.kind === "queue"}
            <QueuePanel
                {queue}
                {settingWallpaper}
                {queueRunning}
                bind:queueIntervalMinutes={queueIntervalMinutes}
                {queueIndex}
                onstartcycling={startCycling}
                onstopcycling={stopCycling}
                onchangeinterval={changeQueueInterval}
                onremove={removeFromQueue}
                onclear={clearQueue}
                onapply={applyWallpaper}
                onreorder={reorderQueue}
            />

        {:else}
            <WallpaperGrid
                {wallpapers}
                {loading}
                {loadingMore}
                {error}
                {hasMore}
                {activeView}
                {queue}
                {settingWallpaper}
                {selectedIndex}
                onapply={applyWallpaper}
                onopenpreview={openPreview}
                ontogglequeue={toggleQueue}
                ondeletehistory={deleteHistoryEntry}
            />
        {/if}
    </main>
</div>

{#if previewWallpaper}
    <PreviewModal
        wallpaper={previewWallpaper}
        tags={previewTags}
        loadingTags={loadingTags}
        onclose={() => (previewWallpaper = null)}
        onapply={(wp) => { applyWallpaper(wp); previewWallpaper = null; }}
        onsearchtag={searchTag}
        onsearchsimilar={searchSimilar}
    />
{/if}

<style>
    :global(body) {
        overflow: hidden;
    }
</style>
