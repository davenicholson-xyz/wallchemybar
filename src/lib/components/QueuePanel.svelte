<script lang="ts">
    import type { Wallpaper } from "$lib/types";

    interface Props {
        queue: Wallpaper[];
        settingWallpaper: string;
        queueRunning: boolean;
        queueIntervalMinutes: number;  // $bindable — two-way sync with input
        queueIndex: number;
        onstartcycling: () => void;
        onstopcycling: () => void;
        onchangeinterval: (minutes: number) => void;
        onremove: (wp: Wallpaper) => void;
        onclear: () => void;
        onapply: (wp: Wallpaper) => void;
        onreorder: (reordered: Wallpaper[]) => void;
    }

    let {
        queue,
        settingWallpaper,
        queueRunning,
        queueIntervalMinutes = $bindable(30),
        queueIndex,
        onstartcycling,
        onstopcycling,
        onchangeinterval,
        onremove,
        onclear,
        onapply,
        onreorder,
    }: Props = $props();
    let dragFromIndex: number | null = $state(null);
    let dragOverIndex: number | null = $state(null);
    let queueDragging = $state(false);

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
            onreorder(reordered);
        } else {
            console.log(`[queue] drop skipped (same position or no source)`);
        }
        dragFromIndex = null;
        dragOverIndex = null;
        queueDragging = false;
    }
</script>

<div class="flex flex-col h-full">
    <div class="flex items-center gap-2.5 px-2.5 py-2 border-b border-base-300 flex-shrink-0">
        <button
            class="btn btn-sm flex-shrink-0 gap-1.5 {queueRunning ? 'btn-error' : 'btn-primary'}"
            onclick={() => queueRunning ? onstopcycling() : onstartcycling()}
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
                onchange={() => { if (queueIntervalMinutes > 0) onchangeinterval(queueIntervalMinutes); }}
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
                        onclick={() => onapply(wp)}
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
                        onclick={(e) => { e.stopPropagation(); onremove(wp); }}
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
            <button class="btn btn-error btn-outline btn-xs" onclick={onclear}>Clear Queue</button>
        </div>
    {/if}
</div>

<style>
    .thumb-wrapper:hover {
        transform: scale(1.03);
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
        z-index: 1;
    }

    .thumb-wrapper:hover .thumb img {
        filter: brightness(1.1) saturate(1.15);
    }

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
