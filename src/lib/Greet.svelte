<script lang="ts">
  import { invoke } from "@tauri-apps/api/core"
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

  let name = "";
  let greetMsg = ""

  async function greet(){
    console.log("WOW")
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke("open_docs", { name })

    const webview = new WebviewWindow('theUniqueLabel', {
  url: 'https://tauri.app/',
})
webview.once('tauri://created', function () {
  console.log("Created")
})
webview.once('tauri://error', function (e) {
  console.log("Error", e)

})
  }
</script>

<div>
  <form class="row" on:submit|preventDefault={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>
</div>