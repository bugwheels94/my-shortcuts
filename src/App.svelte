<script lang="ts">
  import { onMount } from "svelte";
  import yaml from "js-yaml";
  import { WebviewWindow } from "@tauri-apps/api/window";

  import { readTextFile, BaseDirectory, createDir, writeTextFile } from "@tauri-apps/api/fs";
  import { invoke } from "@tauri-apps/api/tauri";
  // Read the text file in the `$APPCONFIG/app.conf` path
  function handleSubmit() {
    const formData = new FormData(this);
    const config = {};
    for (let field of formData) {

      const [key, value] = field;
      config[key] = value;
    }

    setLocalConfig(config);
  }
  let show = false;
  type Config = { gistId: string; token: string, environment: string };
  const config: Config = { gistId: "", token: "" };
  let jsonFile = null;
  function createWebViewFromJs(icon:Icon, label:string) {
    const webview = new WebviewWindow(label, {
      url: icon.url,
      title: label,
      maximized: true,
    });
    // since the webview window is created asynchronously,
    // Tauri emits the `tauri://created` and `tauri://error` to notify you of the creation response
    webview.once("tauri://created", function () {
      // webview window successfully created
    });
    webview.once("tauri://error", function (e) {
      // an error occurred during webview window creation
    });

  }
  async function openIcon(category: string, icon: Icon, webview?:string) {
    const label =
      category +
      "-" +
      icon.name +
      (icon.allowMultipleInstances === "true" ? (Math.random() + 1).toString(36).substring(7) : "");
      
    await invoke("open_icon", { invokeMessage: icon.url, label, webview: webview || "embedded"  });
  }
  const toJson = <T>(content: string = ""): T => {
    try {
      const v =
        yaml.load(content, {
          schema: yaml.JSON_SCHEMA,
        }) || {};
      return v;
    } catch (e) {
      console.log(e)
      return {} as T;
    }
  };
  const toYaml = (content: Object): string => {
    try {
      return yaml.dump(content);
    } catch (e) {
      return "";
    }
  };
  type Icon = {
    url: string;
    icon: string;
    name: string;
    allowMultipleInstances: string;
  };
  type JsonFile = {
    [icons: string]: Icon[];
  } & {
    settings: {
      variables: {
        name: string;
        value: string;
      }[]
    }
  };

  const getLocalConfig = async (): Promise<Config> => {
    try {
      const contents = toJson<Config>(
        await readTextFile("config.yaml", {
          dir: BaseDirectory.AppData,
        })
      );
      config.gistId = contents.gistId || "";
      config.token = contents.token || "";
      config.environment = contents.environment || "default"

      return contents;
    } catch (e) {
      await createDir("", { dir: BaseDirectory.AppData, recursive: true });
      await writeTextFile("config.yaml", "", {
        dir: BaseDirectory.AppData,
      });
      // return getLocalConfig();
    }
  };

  async function setLocalConfig(partialConfig: Partial<Config>) {
    try {
      const finalConfig = {
        ...config,
        ...partialConfig,
      };
      const yaml = toYaml(finalConfig);
      await writeTextFile("config.yaml", yaml, {
        dir: BaseDirectory.AppData,
      });
      getLocalConfig();
    } catch (e) {
      await createDir("", { dir: BaseDirectory.AppData, recursive: true });
      return setLocalConfig(partialConfig);
    }
  }
  let json: { content: Omit<JsonFile, "settings">; meta?: Config, webview?: string} = {
    content: {},
  };

  $: {
    (async function () {
      if (jsonFile) return;
      if (config.gistId && config.token) {
        const res = await fetch(
          "https://api.github.com/gists/" + config.gistId + "?time=" + new Date(),
          {
            headers: {
              "X-GitHub-Api-Version": "2022-11-28",
              Authorization: "Bearer " + config.token,
              Accept: "application/vnd.github+json",
            },
          }
        ).then((res) => res.json());
        const files = res.files;
        for (let file in files) {
          jsonFile = toJson<JsonFile>(files[file].content);

          break;
        }
      }
    })();
  }
  $: {
    (function(){
      if (!jsonFile) return;
      console.log(jsonFile)
      const environments = jsonFile.settings.environments;
      const variables = (environments[config.environment] || []).concat(environments.default|| [])
        .filter((v,i,a)=>a.findIndex(v2=>(v2.name===v.name))===i);
      json.meta = config;
      const reservedFields = ['settings'];
      json.webview = variables.find(variable => variable.name === "webview").value || ""

      Object.keys(jsonFile).forEach((category) => {
        if (reservedFields.includes(category)) return;
        json.content[category] = jsonFile[category].map((icon) => ({
          allowMultipleInstances: "false", // rust does not support optional
          ...icon,
          url: variables.reduce((url, variable) => {
            return url.replace("$" + variable.name, variable.value);
          }, icon.url),
          icon: variables.reduce((icon, variable) => {
            return icon.replace("$" + variable.name, variable.value);
          }, icon.icon),
        }));
      });
      console.log("woww", json)
    })();
  }
  $: {
    (async function () {
      if (json.meta) {
        await invoke("load_json", { request: json });
      }
    })();
  }
  onMount(async () => {
    await getLocalConfig();
  });
</script>

<main class="container">
  <button on:click={() => (show = !show)}>Configure</button>
  {#if show}
    <form on:submit|preventDefault={handleSubmit}>
      <input name="token" type="password" value={config.token} />
      <input name="gistId" value={config.gistId} />
      <input name="environment" value={config.environment} />
      <button type="submit"> Save </button>
    </form>
  {/if}

  {#each Object.keys(json.content) as category}
    <div class="icons">
      <div class="title">
        {category}
      </div>
      {#each json.content[category] as icon}
        <button
          class="icon"
          on:click={() => openIcon(category, icon, json.webview)}
          on:keyup={(e) => e.key === "Enter" && openIcon(category, icon, json.webview)}
        >
          <img src={icon.icon} alt="icon.url" />
          {icon.name || "unnamed"}
        </button>
      {/each}
    </div>
  {/each}

  <div class="row" />
</main>

<style>
  :global(html) {
    font-size: 62.5% !important;
  }
  .title {
    padding: 1rem;
    margin: 0 1rem;
    flex-basis: 100%;
    font-weight: bold;
    border-bottom: solid 1px #efefef;
  }
  .icons {
    border: solid 1px #ccc;
    display: flex;
    justify-content: flex-start;
    font-size: 1.2rem;
    line-height: 1.4rem;
    flex-wrap: wrap;
    margin: 1rem 0;
  }
  .icon {
    border: solid 1px #ccc;
    padding: 1rem 2rem;
    cursor: pointer;
    max-width: 6rem;
    display: flex;
    justify-content: center;
    margin: 1rem;
    flex-direction: column;
    align-items: center;
  }
  .icon:focus {
    border: dashed 1px #000;
  }
  .icon > img {
    display: block;
    max-width: 2rem;
    margin-bottom: 0.5rem;
  }
</style>
