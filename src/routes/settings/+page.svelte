<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";

    let username = $state("");
    let apiKey = $state("");
    let sfw = $state(true);
    let sketchy = $state(false);
    let nsfw = $state(false);
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
            atleast: string;
        } = await invoke("load_settings");
        username = settings.username;
        apiKey = settings.api_key;
        sfw = settings.purity[0] === "1";
        sketchy = settings.purity[1] === "1";
        nsfw = settings.purity[2] === "1";
        atleast = settings.atleast ?? "";
    });

    function getPurity(): string {
        return `${sfw ? "1" : "0"}${sketchy ? "1" : "0"}${nsfw ? "1" : "0"}`;
    }

    async function save() {
        await invoke("save_settings", {
            settings: {
                username,
                api_key: apiKey,
                purity: getPurity(),
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
            Username
            <input
                type="text"
                bind:value={username}
                placeholder="wallhaven username"
            />
        </label>
        <label>
            API Key
            <input
                type="password"
                bind:value={apiKey}
                placeholder="wallhaven API key"
            />
        </label>
        <fieldset>
            <legend>Purity</legend>
            <label class="check">
                <input type="checkbox" bind:checked={sfw} />
                SFW
            </label>
            <label class="check">
                <input type="checkbox" bind:checked={sketchy} />
                Sketchy
            </label>
            <label class="check">
                <input type="checkbox" bind:checked={nsfw} />
                NSFW
            </label>
        </fieldset>
        <label>
            Minimum Resolution
            <select bind:value={atleast}>
                <option value="">Any</option>
                {#each availableResolutions as res}
                    <option value={res}>{res}</option>
                {/each}
            </select>
        </label>
        <button type="submit">Save</button>
    </form>
</main>

<style>
    :root {
        font-family:
            -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        font-size: 14px;
        color: #0f0f0f;
        background-color: #f6f6f6;
    }

    main {
        padding: 20px;
    }

    h2 {
        margin: 0 0 16px 0;
        font-size: 18px;
    }

    form {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    label {
        display: flex;
        flex-direction: column;
        gap: 4px;
        font-weight: 500;
        font-size: 13px;
    }

    input {
        padding: 8px 10px;
        border: 1px solid #ccc;
        border-radius: 6px;
        font-size: 14px;
        background: #fff;
        color: #0f0f0f;
    }

    input:focus {
        outline: none;
        border-color: #646cff;
    }

    button {
        margin-top: 4px;
        padding: 8px 16px;
        border: none;
        border-radius: 6px;
        background: #646cff;
        color: #fff;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
    }

    button:hover {
        background: #535bf2;
    }

    fieldset {
        border: 1px solid #ccc;
        border-radius: 6px;
        padding: 8px 12px;
        margin: 0;
    }

    legend {
        font-weight: 500;
        font-size: 13px;
        padding: 0 4px;
    }

    .check {
        flex-direction: row;
        align-items: center;
        gap: 6px;
        font-weight: normal;
        cursor: pointer;
    }

    .check input {
        margin: 0;
        padding: 0;
        border: none;
        box-shadow: none;
    }

    select {
        padding: 8px 10px;
        border: 1px solid #ccc;
        border-radius: 6px;
        font-size: 14px;
        background: #fff;
        color: #0f0f0f;
    }

    select:focus {
        outline: none;
        border-color: #646cff;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            color: #f6f6f6;
            background-color: #2f2f2f;
        }

        input,
        select {
            background: #1a1a1a;
            border-color: #444;
            color: #f6f6f6;
        }

        fieldset {
            border-color: #444;
        }

        button {
            background: #646cff;
        }

        button:hover {
            background: #535bf2;
        }
    }
</style>
