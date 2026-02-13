<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";

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
        "1280x720",
        "1280x800",
        "1280x960",
        "1280x1024",
        "1600x900",
        "1600x1000",
        "1600x1200",
        "1600x1280",
        "1920x1080",
        "1920x1200",
        "1920x1440",
        "1920x1536",
        "2560x1080",
        "2560x1440",
        "2560x1600",
        "2560x1920",
        "2560x2048",
        "3440x1440",
        "3840x1600",
        "3840x2160",
        "3840x2400",
        "3840x2880",
        "3840x3072",
    ];

    onMount(async () => {
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
    });

    function getPurity(): string {
        return `${sfw ? "1" : "0"}${sketchy ? "1" : "0"}${nsfw ? "1" : "0"}`;
    }

    function getCategories(): string {
        return `${general ? "1" : "0"}${anime ? "1" : "0"}${people ? "1" : "0"}`;
    }

    async function clearHistory() {
        await invoke("clear_history");
    }

    async function save() {
        await invoke("save_settings", {
            settings: {
                username,
                api_key: apiKey,
                purity: getPurity(),
                categories: getCategories(),
                atleast,
            },
        });
        getCurrentWindow().hide();
    }
</script>

<main>
    <h2>Settings</h2>
    <form onsubmit={save}>
        <label>
            <span class="label-text">Username</span>
            <input
                type="text"
                bind:value={username}
                placeholder="wallhaven username"
            />
        </label>
        <label>
            <span class="label-text">API Key</span>
            <input
                type="password"
                bind:value={apiKey}
                placeholder="wallhaven API key"
            />
        </label>

        <div class="field">
            <span class="label-text">Categories</span>
            <div class="purity-group">
                <button
                    type="button"
                    class="purity-btn category"
                    class:active={general}
                    onclick={() => (general = !general)}>General</button
                >
                <button
                    type="button"
                    class="purity-btn category"
                    class:active={anime}
                    onclick={() => (anime = !anime)}>Anime</button
                >
                <button
                    type="button"
                    class="purity-btn category"
                    class:active={people}
                    onclick={() => (people = !people)}>People</button
                >
            </div>
        </div>

        <div class="field">
            <span class="label-text">Purity</span>
            <div class="purity-group">
                <button
                    type="button"
                    class="purity-btn sfw"
                    class:active={sfw}
                    onclick={() => (sfw = !sfw)}>SFW</button
                >
                <button
                    type="button"
                    class="purity-btn sketchy"
                    class:active={sketchy}
                    onclick={() => (sketchy = !sketchy)}>Sketchy</button
                >
                <button
                    type="button"
                    class="purity-btn nsfw"
                    class:active={nsfw}
                    onclick={() => (nsfw = !nsfw)}>NSFW</button
                >
            </div>
        </div>

        <label>
            <span class="label-text">Minimum Resolution</span>
            <select bind:value={atleast}>
                <option value="">Any</option>
                {#each availableResolutions as res}
                    <option value={res}>{res}</option>
                {/each}
            </select>
        </label>

        <button class="save-btn" type="submit">Save</button>
    </form>

    <div class="danger-zone">
        <button class="clear-history-btn" onclick={clearHistory}
            >Clear History</button
        >
    </div>
</main>

<style>
    :global(body) {
        overflow: hidden;
    }

    :root {
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        font-size: 14px;
        color: #e0e0e0;
        background: linear-gradient(
            145deg,
            #1a1a2e 0%,
            #16213e 50%,
            #1a1a2e 100%
        );
    }

    main {
        padding: 24px;
        min-height: 100vh;
        box-sizing: border-box;
    }

    h2 {
        margin: 0 0 20px 0;
        font-size: 20px;
        font-weight: 600;
        color: #fff;
    }

    form {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    label {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .label-text {
        font-weight: 500;
        font-size: 13px;
        color: #aaa;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    input,
    select {
        padding: 10px 12px;
        border: 1px solid #333;
        border-radius: 8px;
        font-size: 14px;
        background: rgba(255, 255, 255, 0.06);
        color: #e0e0e0;
        transition:
            border-color 0.2s,
            background 0.2s;
    }

    input:focus,
    select:focus {
        outline: none;
        border-color: #646cff;
        background: rgba(255, 255, 255, 0.1);
    }

    input::placeholder {
        color: #555;
    }

    select option {
        background: #1a1a2e;
        color: #e0e0e0;
    }

    /* Purity toggle buttons - wallhaven style */
    .purity-group {
        display: flex;
        gap: 8px;
    }

    .purity-btn {
        flex: 1;
        padding: 8px 16px;
        border: 2px solid transparent;
        border-radius: 6px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        color: #888;
        background: rgba(255, 255, 255, 0.06);
    }

    .purity-btn:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    .purity-btn.sfw.active {
        background: linear-gradient(135deg, #4a6741 0%, #3d5a35 100%);
        color: #d4e6cf;
        border-color: #5a7d4f;
    }

    .purity-btn.sketchy.active {
        background: linear-gradient(135deg, #8a7a3a 0%, #6e622e 100%);
        color: #e6ddb3;
        border-color: #9e8d42;
    }

    .purity-btn.nsfw.active {
        background: linear-gradient(135deg, #7a3a3a 0%, #622e2e 100%);
        color: #e6b3b3;
        border-color: #9e4242;
    }

    .danger-zone {
        margin-top: 16px;
        padding-top: 16px;
        border-top: 1px solid #333;
    }

    .clear-history-btn {
        width: 100%;
        padding: 10px 20px;
        border: 2px solid #7a3a3a;
        border-radius: 8px;
        background: rgba(122, 58, 58, 0.15);
        color: #e6b3b3;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        transition:
            background 0.2s,
            border-color 0.2s;
    }

    .clear-history-btn:hover {
        background: rgba(122, 58, 58, 0.3);
        border-color: #9e4242;
    }

    .purity-btn.category.active {
        background: linear-gradient(135deg, #3a5a7a 0%, #2e4a62 100%);
        color: #b3d4e6;
        border-color: #4278a0;
    }

    /* Save button */
    .save-btn {
        margin-top: 8px;
        padding: 10px 20px;
        border: none;
        border-radius: 8px;
        background: linear-gradient(135deg, #646cff 0%, #535bf2 100%);
        color: #fff;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        transition:
            opacity 0.2s,
            transform 0.1s;
    }

    .save-btn:hover {
        opacity: 0.9;
    }

    .save-btn:active {
        transform: scale(0.98);
    }
</style>
