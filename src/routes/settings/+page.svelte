<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let username = $state("");
  let apiKey = $state("");

  onMount(async () => {
    const settings: { username: string; api_key: string } =
      await invoke("load_settings");
    username = settings.username;
    apiKey = settings.api_key;
  });

  async function save() {
    await invoke("save_settings", {
      settings: { username, api_key: apiKey },
    });
    getCurrentWindow().hide();
  }
</script>

<main>
  <h2>Settings</h2>
  <form onsubmit={save}>
    <label>
      Username
      <input type="text" bind:value={username} placeholder="wallhaven username" />
    </label>
    <label>
      API Key
      <input type="password" bind:value={apiKey} placeholder="wallhaven API key" />
    </label>
    <button type="submit">Save</button>
  </form>
</main>

<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
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

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    input {
      background: #1a1a1a;
      border-color: #444;
      color: #f6f6f6;
    }

    button {
      background: #646cff;
    }

    button:hover {
      background: #535bf2;
    }
  }
</style>
