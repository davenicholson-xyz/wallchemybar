<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, tick } from "svelte";
    import type { Tag, Wallpaper, Collection, View } from "$lib/types";
    import ExpandedSidebar from "$lib/components/ExpandedSidebar.svelte";
    import WallpaperGrid from "$lib/components/WallpaperGrid.svelte";
    import QueuePanel from "$lib/components/QueuePanel.svelte";
    import SettingsPanel from "$lib/components/SettingsPanel.svelte";
    import PreviewPanel from "$lib/components/PreviewPanel.svelte";

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
    let selectedIndex = $state(-1);
    let hoverWallpaper: Wallpaper | null = $state(null);

    let isGridView = $derived(
        activeView.kind !== "settings" && activeView.kind !== "queue"
    );

    $effect(() => {
        if (loading) { selectedIndex = -1; previewTags = []; }
    });

    // Auto-fill: after each page loads, if the content doesn't yet overflow the
    // scroll container fetch another page — keeps going until the user can scroll.
    $effect(() => {
        const _ = wallpapers.length;
        if (loading || loadingMore || !hasMore || !isGridView) return;
        tick().then(() => {
            if (mainEl && mainEl.scrollHeight <= mainEl.clientHeight) {
                loadNextPage();
            }
        });
    });

    let undoing = $state(false);

    // ─── Preview state (for PreviewPanel) ────────────────────────────────────────
    let previewTags: Tag[] = $state([]);
    let loadingTags = $state(false);
    let previewWallpaper: Wallpaper | null = $derived(
        hoverWallpaper ?? (selectedIndex >= 0 ? wallpapers[selectedIndex] ?? null : null)
    );

    // Tag cache: avoids re-fetching wallpapers already seen this session.
    // Tags are only loaded when a thumb is clicked — hover never triggers an API request.
    const tagCache = new Map<string, Tag[]>();

    async function loadTagsFor(wp: Wallpaper) {
        const cached = tagCache.get(wp.id);
        if (cached) {
            previewTags = cached;
            return;
        }
        previewTags = [];
        loadingTags = true;
        try {
            const tags = await invoke<Tag[]>("fetch_wallpaper_tags", { wallpaperId: wp.id });
            tagCache.set(wp.id, tags);
            if (previewWallpaper?.id === wp.id) previewTags = tags;
        } catch {}
        if (previewWallpaper?.id === wp.id) loadingTags = false;
    }

    // ─── Queue state ─────────────────────────────────────────────────────────────
    let queue: Wallpaper[] = $state([]);
    let queueIntervalMinutes = $state(30);
    let queueRunning = $state(false);
    let queueIndex = $state(0);
    let queueTimerId: ReturnType<typeof setInterval> | undefined;

    // ─── Lifecycle ───────────────────────────────────────────────────────────────
    onMount(async () => {
        invoke("fetch_collections")
            .then((cols) => { collections = cols as Collection[]; })
            .catch(() => {});
        await loadSearch("hot");
    });

    // ─── Search / browse ─────────────────────────────────────────────────────────
    async function loadSearch(sorting: string) {
        activeView = { kind: "search", sorting };
        page = 1;
        hasMore = true;
        loading = true;
        error = "";
        try {
            const results: Wallpaper[] = await invoke("fetch_search", { sorting, page: 1 });
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
        selectedCollectionId = id;
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

    async function loadNextPage() {
        if (loadingMore || !hasMore) return;
        if (activeView.kind === "history") return;
        loadingMore = true;
        const nextPage = page + 1;
        try {
            let results: Wallpaper[];
            if (activeView.kind === "search") {
                results = await invoke("fetch_search", { sorting: activeView.sorting, page: nextPage });
            } else if (activeView.kind === "query") {
                results = await invoke("fetch_search", { sorting: "relevance", page: nextPage, query: activeView.query });
            } else if (activeView.kind === "collection") {
                results = await invoke("fetch_collection_wallpapers", { collectionId: activeView.id, page: nextPage });
            } else {
                loadingMore = false;
                return;
            }
            const existingIds = new Set(wallpapers.map((w) => w.id));
            const newResults = results.filter((w) => !existingIds.has(w.id));
            wallpapers = [...wallpapers, ...newResults];
            page = nextPage;
            hasMore = results.length >= 24;
        } catch {
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
            if (scrollHeight - scrollTop - clientHeight < 200) loadNextPage();
        });
    }

    // ─── Wallpaper actions ───────────────────────────────────────────────────────
    async function applyWallpaper(wp: Wallpaper) {
        // Expanded window stays open after apply (no hide_main call)
        settingWallpaper = wp.id;
        try {
            await invoke("set_wallpaper", { wallpaper: wp });
        } catch (e) {
            error = String(e);
        } finally {
            settingWallpaper = "";
        }
    }

    async function undoWallpaper() {
        undoing = true;
        try {
            await invoke("undo_wallpaper");
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

    // ─── Preview panel search actions ────────────────────────────────────────────
    async function searchTag(tag: Tag) {
        hoverWallpaper = null;
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
        hoverWallpaper = null;
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
        if (queueRunning) {
            clearInterval(queueTimerId);
            queueTimerId = setInterval(advanceQueue, minutes * 60 * 1000);
        }
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
        invoke("reorder_queue", { wallpapers: reordered }).catch((err) => { error = String(err); });
    }

    function openSettings() {
        activeView = { kind: "settings" };
    }

    function handleKeyDown(e: KeyboardEvent) {
        const target = e.target as HTMLElement;
        const inInput = target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT";

        if (e.key === "Escape") {
            hoverWallpaper = null;
            return;
        }

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
                case "E": e.preventDefault(); invoke("close_expanded"); return;
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
                selectedIndex = selectedIndex < 0 ? 0 : Math.min(selectedIndex + 5, wallpapers.length - 1);
                break;
            case "ArrowUp":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.max(selectedIndex - 5, 0);
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
                selectedIndex = selectedIndex < 0 ? 0 : Math.min(selectedIndex + 5, wallpapers.length - 1);
                break;
            case "k":
                e.preventDefault();
                selectedIndex = selectedIndex < 0 ? 0 : Math.max(selectedIndex - 5, 0);
                break;
            case " ":
                if (selectedIndex >= 0 && wallpapers[selectedIndex]) {
                    e.preventDefault();
                    // Space focuses the selected wallpaper in preview — already handled by derived
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
        <ExpandedSidebar
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
            oncollapsetopopup={() => invoke("close_expanded")}
        />

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
                    cols={5}
                    onhoverwallpaper={(wp) => { hoverWallpaper = wp; }}
                    onapply={(wp) => { selectedIndex = wallpapers.findIndex((w) => w.id === wp.id); hoverWallpaper = null; loadTagsFor(wp); }}
                    onopenpreview={(wp) => { selectedIndex = wallpapers.findIndex((w) => w.id === wp.id); hoverWallpaper = null; loadTagsFor(wp); }}
                    ontogglequeue={toggleQueue}
                    ondeletehistory={deleteHistoryEntry}
                />
            {/if}
        </main>

        <PreviewPanel
            wallpaper={previewWallpaper}
            tags={previewTags}
            {loadingTags}
            onapply={applyWallpaper}
            onsearchtag={searchTag}
            onsearchsimilar={searchSimilar}
        />
</div>

<style>
    :global(body) {
        overflow: hidden;
    }
</style>
