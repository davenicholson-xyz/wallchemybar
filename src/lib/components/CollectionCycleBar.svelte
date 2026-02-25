<script lang="ts">
    interface Props {
        cycling: boolean;
        intervalMinutes: number;
        onstart: () => void;
        onstop: () => void;
        onchangeinterval: (minutes: number) => void;
    }

    let { cycling, intervalMinutes = $bindable(30), onstart, onstop, onchangeinterval }: Props = $props();
</script>

<div class="flex items-center gap-2.5 px-0.5 py-1">
    <button
        class="btn btn-sm flex-shrink-0 gap-1.5 {cycling ? 'btn-error' : 'btn-primary'}"
        onclick={() => cycling ? onstop() : onstart()}
        title={cycling ? "Stop collection cycling" : "Cycle this collection"}
    >
        {#if cycling}
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
            Stop
        {:else}
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
            Cycle
        {/if}
    </button>
    <div class="flex items-center gap-1.5">
        <span class="text-[10px] text-base-content/40 whitespace-nowrap">Every</span>
        <input
            type="number"
            class="input input-xs input-bordered w-14 bg-base-200 border-base-300 text-center [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
            min="1"
            max="1440"
            bind:value={intervalMinutes}
            onchange={() => { if (intervalMinutes > 0) onchangeinterval(intervalMinutes); }}
        />
        <span class="text-[10px] text-base-content/40 whitespace-nowrap">min</span>
    </div>
    {#if cycling}
        <span class="ml-auto w-2 h-2 rounded-full bg-primary animate-pulse flex-shrink-0" title="Collection cycling active"></span>
    {/if}
</div>
