<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    interface Props {
        onreloadsearch: (sorting: string) => void;
    }

    let { onreloadsearch }: Props = $props();

    let username = $state("");
    let apiKey = $state("");
    let sfw = $state(true);
    let sketchy = $state(false);
    let nsfw = $state(false);
    let general = $state(true);
    let anime = $state(true);
    let people = $state(true);
    let atleast = $state("");

    const availableResolutions = [
        "1280x720", "1280x800", "1280x960", "1280x1024",
        "1600x900", "1600x1000", "1600x1200", "1600x1280",
        "1920x1080", "1920x1200", "1920x1440", "1920x1536",
        "2560x1080", "2560x1440", "2560x1600", "2560x1920", "2560x2048",
        "3440x1440", "3840x1600", "3840x2160", "3840x2400", "3840x2880", "3840x3072",
    ];

    onMount(async () => {
        try {
            const settings: {
                username: string;
                api_key: string;
                purity: string;
                categories: string;
                atleast: string;
            } = await invoke("load_settings");
            username = settings.username;
            apiKey = settings.api_key;
            sfw = settings.purity[0] === "1";
            sketchy = settings.purity[1] === "1";
            nsfw = settings.purity[2] === "1";
            const cats = settings.categories ?? "111";
            general = cats[0] === "1";
            anime = cats[1] === "1";
            people = cats[2] === "1";
            atleast = settings.atleast ?? "";
        } catch {}
    });

    async function save() {
        const purity = `${sfw ? "1" : "0"}${sketchy ? "1" : "0"}${nsfw ? "1" : "0"}`;
        const categories = `${general ? "1" : "0"}${anime ? "1" : "0"}${people ? "1" : "0"}`;
        await invoke("save_settings", {
            settings: { username, api_key: apiKey, purity, categories, atleast },
        });
        onreloadsearch("hot");
    }

    async function clearHistory() {
        await invoke("clear_history");
    }
</script>

<div class="p-[18px] h-full box-border overflow-y-auto">
    <h3 class="mt-0 mb-4 text-[15px] font-semibold text-base-content tracking-[0.3px]">Settings</h3>
    <form class="flex flex-col gap-3.5" onsubmit={(e) => { e.preventDefault(); save(); }}>
        <div class="flex flex-col gap-1">
            <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Username</span>
            <input type="text" class="input input-sm input-bordered bg-base-200 border-base-300 placeholder:text-base-content/20" bind:value={username} placeholder="wallhaven username" />
        </div>
        <div class="flex flex-col gap-1">
            <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">API Key</span>
            <input type="password" class="input input-sm input-bordered bg-base-200 border-base-300 placeholder:text-base-content/20" bind:value={apiKey} placeholder="wallhaven API key" />
        </div>
        <div class="flex flex-col gap-1">
            <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Categories</span>
            <div class="join w-full">
                <button type="button" class="join-item btn btn-sm flex-1 border-base-300 {general ? 'bg-info/20 text-info border-info/40' : 'bg-base-200 text-base-content/30'}" onclick={() => (general = !general)}>General</button>
                <button type="button" class="join-item btn btn-sm flex-1 border-base-300 {anime ? 'bg-info/20 text-info border-info/40' : 'bg-base-200 text-base-content/30'}" onclick={() => (anime = !anime)}>Anime</button>
                <button type="button" class="join-item btn btn-sm flex-1 border-base-300 {people ? 'bg-info/20 text-info border-info/40' : 'bg-base-200 text-base-content/30'}" onclick={() => (people = !people)}>People</button>
            </div>
        </div>
        <div class="flex flex-col gap-1">
            <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Purity</span>
            <div class="join w-full">
                <button type="button" class="join-item btn btn-sm flex-1 border-base-300 {sfw ? 'bg-success/20 text-success border-success/40' : 'bg-base-200 text-base-content/30'}" onclick={() => (sfw = !sfw)}>SFW</button>
                <button type="button" class="join-item btn btn-sm flex-1 border-base-300 {sketchy ? 'bg-warning/20 text-warning border-warning/40' : 'bg-base-200 text-base-content/30'}" onclick={() => (sketchy = !sketchy)}>Sketchy</button>
                <button type="button" class="join-item btn btn-sm flex-1 border-base-300 {nsfw ? 'bg-error/20 text-error border-error/40' : 'bg-base-200 text-base-content/30'}" onclick={() => (nsfw = !nsfw)}>NSFW</button>
            </div>
        </div>
        <div class="flex flex-col gap-1">
            <span class="text-[10px] font-semibold text-base-content/40 uppercase tracking-[0.8px]">Minimum Resolution</span>
            <select class="select select-sm select-bordered bg-base-200 border-base-300" bind:value={atleast}>
                <option value="">Any</option>
                {#each availableResolutions as res}
                    <option value={res}>{res}</option>
                {/each}
            </select>
        </div>
        <button class="btn btn-primary btn-sm mt-1" type="submit">Save</button>
    </form>
    <div class="mt-3.5 pt-3.5 border-t border-base-300 flex flex-col gap-2">
        <button class="btn btn-error btn-outline btn-sm w-full" onclick={clearHistory}>Clear History</button>
        <button class="btn btn-ghost btn-sm w-full text-base-content/40" onclick={() => invoke("quit_app")}>Quit</button>
    </div>
</div>
