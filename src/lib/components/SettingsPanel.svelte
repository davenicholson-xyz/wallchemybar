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
    let linuxWallpaperCmd = $state("");
    let isLinux = $state(false);

    const ALL_RATIOS = ["16x9", "16x10", "21x9", "9x16", "4x3"] as const;
    type Ratio = typeof ALL_RATIOS[number];
    let selectedRatios = $state(new Set<Ratio>());

    type ApiKeyStatus = "idle" | "checking" | "valid" | "invalid";
    let apiKeyStatus = $state<ApiKeyStatus>("idle");

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
                ratios: string;
                linux_wallpaper_cmd: string;
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
            linuxWallpaperCmd = settings.linux_wallpaper_cmd ?? "";
            isLinux = await invoke<boolean>("is_linux");
            selectedRatios = new Set(
                (settings.ratios ?? "").split(",").filter(r => r) as Ratio[]
            );
            if (settings.api_key.trim()) {
                validateKey(settings.api_key);
            }
        } catch {}
    });

    async function validateKey(key: string) {
        apiKeyStatus = "checking";
        try {
            const valid = await invoke<boolean>("validate_api_key", { apiKey: key });
            apiKeyStatus = valid ? "valid" : "invalid";
        } catch {
            apiKeyStatus = "idle";
        }
    }

    async function save() {
        const purity = `${sfw ? "1" : "0"}${sketchy ? "1" : "0"}${nsfw ? "1" : "0"}`;
        const categories = `${general ? "1" : "0"}${anime ? "1" : "0"}${people ? "1" : "0"}`;
        await invoke("save_settings", {
            settings: { username, api_key: apiKey, purity, categories, atleast, ratios: Array.from(selectedRatios).join(","), linux_wallpaper_cmd: linuxWallpaperCmd },
        });
        onreloadsearch("hot");
        if (apiKey.trim()) {
            validateKey(apiKey);
        } else {
            apiKeyStatus = "idle";
        }
    }

    async function clearHistory() {
        await invoke("clear_history");
    }
</script>

<div class="p-[14px] h-full box-border overflow-y-auto flex flex-col gap-3">

    <h3 class="mt-0 mb-0 text-[13px] font-semibold text-base-content/80 tracking-[0.2px]">Settings</h3>

    <!-- Account -->
    <div class="flex flex-col gap-[5px]">
        <span class="text-[9px] font-semibold text-base-content/25 uppercase tracking-[1.2px] px-[2px]">Account</span>
        <div class="bg-base-200 rounded-lg overflow-hidden">
            <div class="flex items-center gap-2.5 px-3 py-2.5">
                <span class="text-[11px] text-base-content/40 w-[62px] shrink-0">Username</span>
                <input
                    type="text"
                    class="flex-1 min-w-0 bg-transparent border-none outline-none text-[12px] text-base-content placeholder:text-base-content/20"
                    bind:value={username}
                    placeholder="wallhaven username"
                />
            </div>
            <div class="border-t border-base-300/50 flex items-center gap-2.5 px-3 py-2.5">
                <div class="flex items-center gap-1.5 w-[62px] shrink-0">
                    <span class="text-[11px] text-base-content/40">API Key</span>
                    {#if apiKeyStatus === "checking"}
                        <span class="loading loading-spinner text-base-content/25" style="width:7px;height:7px"></span>
                    {:else if apiKeyStatus === "valid"}
                        <span class="w-[7px] h-[7px] rounded-full bg-success shrink-0" title="Valid"></span>
                    {:else if apiKeyStatus === "invalid"}
                        <span class="w-[7px] h-[7px] rounded-full bg-error shrink-0" title="Invalid"></span>
                    {/if}
                </div>
                <input
                    type="password"
                    class="flex-1 min-w-0 bg-transparent border-none outline-none text-[12px] text-base-content placeholder:text-base-content/20"
                    bind:value={apiKey}
                    oninput={() => { apiKeyStatus = "idle"; }}
                    placeholder="api key"
                />
            </div>
        </div>
    </div>

    <!-- Filters -->
    <div class="flex flex-col gap-[5px]">
        <span class="text-[9px] font-semibold text-base-content/25 uppercase tracking-[1.2px] px-[2px]">Filters</span>
        <div class="bg-base-200 rounded-lg overflow-hidden">
            <div class="flex items-center gap-2.5 px-3 py-2">
                <span class="text-[11px] text-base-content/40 w-[62px] shrink-0">Purity</span>
                <div class="join flex-1">
                    <button type="button" class="join-item btn btn-xs flex-1 {sfw ? 'bg-success/20 text-success border-success/30' : 'bg-base-300/60 text-base-content/30 border-transparent'}" onclick={() => (sfw = !sfw)}>SFW</button>
                    <button type="button" class="join-item btn btn-xs flex-1 {sketchy ? 'bg-warning/20 text-warning border-warning/30' : 'bg-base-300/60 text-base-content/30 border-transparent'}" onclick={() => (sketchy = !sketchy)}>Sketchy</button>
                    <button type="button" class="join-item btn btn-xs flex-1 {nsfw ? 'bg-error/20 text-error border-error/30' : 'bg-base-300/60 text-base-content/30 border-transparent'}" onclick={() => (nsfw = !nsfw)}>NSFW</button>
                </div>
            </div>
            <div class="border-t border-base-300/50 flex items-center gap-2.5 px-3 py-2">
                <span class="text-[11px] text-base-content/40 w-[62px] shrink-0">Categories</span>
                <div class="join flex-1">
                    <button type="button" class="join-item btn btn-xs flex-1 {general ? 'bg-info/20 text-info border-info/30' : 'bg-base-300/60 text-base-content/30 border-transparent'}" onclick={() => (general = !general)}>General</button>
                    <button type="button" class="join-item btn btn-xs flex-1 {anime ? 'bg-info/20 text-info border-info/30' : 'bg-base-300/60 text-base-content/30 border-transparent'}" onclick={() => (anime = !anime)}>Anime</button>
                    <button type="button" class="join-item btn btn-xs flex-1 {people ? 'bg-info/20 text-info border-info/30' : 'bg-base-300/60 text-base-content/30 border-transparent'}" onclick={() => (people = !people)}>People</button>
                </div>
            </div>
            <div class="border-t border-base-300/50 flex items-center gap-2.5 px-3 py-2">
                <span class="text-[11px] text-base-content/40 w-[62px] shrink-0">Ratio</span>
                <div class="flex flex-wrap gap-1 flex-1">
                    {#each ALL_RATIOS as ratio}
                        {@const active = selectedRatios.has(ratio)}
                        <button
                            type="button"
                            class="btn btn-xs border px-2 {active ? 'bg-primary/20 text-primary border-primary/30' : 'bg-base-300/60 text-base-content/30 border-transparent'}"
                            onclick={() => {
                                const next = new Set(selectedRatios);
                                next.has(ratio) ? next.delete(ratio) : next.add(ratio);
                                selectedRatios = next;
                            }}
                        >{ratio.replace("x", ":")}</button>
                    {/each}
                </div>
            </div>
            <div class="border-t border-base-300/50 flex items-center gap-2.5 px-3 py-2.5">
                <span class="text-[11px] text-base-content/40 w-[62px] shrink-0">Resolution</span>
                <select
                    class="flex-1 min-w-0 bg-transparent border-none outline-none text-[12px] text-base-content cursor-pointer"
                    bind:value={atleast}
                >
                    <option value="">Any</option>
                    {#each availableResolutions as res}
                        <option value={res}>{res}</option>
                    {/each}
                </select>
            </div>
        </div>
    </div>

    <!-- Linux wallpaper command -->
    {#if isLinux}
    <div class="flex flex-col gap-[5px]">
        <span class="text-[9px] font-semibold text-base-content/25 uppercase tracking-[1.2px] px-[2px]">Linux</span>
        <div class="bg-base-200 rounded-lg overflow-hidden">
            <div class="flex items-center gap-2.5 px-3 py-2.5">
                <span class="text-[11px] text-base-content/40 w-[62px] shrink-0">Wallpaper cmd</span>
                <input
                    type="text"
                    class="flex-1 min-w-0 bg-transparent border-none outline-none text-[12px] text-base-content placeholder:text-base-content/20"
                    bind:value={linuxWallpaperCmd}
                    placeholder="e.g. feh --bg-fill"
                />
            </div>
        </div>
    </div>
    {/if}

    <button class="btn btn-primary btn-sm w-full" onclick={save}>Save</button>

    <div class="mt-auto pt-3 border-t border-base-300/50 flex flex-col gap-1.5">
        <button class="btn btn-error btn-outline btn-xs w-full" onclick={clearHistory}>Clear History</button>
        <button class="btn btn-ghost btn-xs w-full text-base-content/30" onclick={() => invoke("quit_app")}>Quit</button>
    </div>
</div>
