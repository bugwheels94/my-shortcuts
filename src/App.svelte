<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import yaml from "js-yaml";
  import {
    readTextFile,
    BaseDirectory,
    createDir,
    writeTextFile,
  } from "@tauri-apps/api/fs";
  // Read the text file in the `$APPCONFIG/app.conf` path
  function handleSubmit() {
    const formData = new FormData(this);
    const config = {};
    for (let field of formData) {
      console.log(field);

      const [key, value] = field;
      config[key] = value;
    }

    setLocalConfig(config);
  }

  let show = false;
  type Config = { gistId: string; token: string };
  const config: Config = { gistId: "", token: "" };
  async function openIcon(url) {
    await invoke("open_docs", { invokeMessage: url });
  }
  const toJson = <T>(content: string = ""): T => {
    try {
      const v =
        yaml.load(content, {
          schema: yaml.JSON_SCHEMA,
        }) || {};
      return v;
    } catch (e) {
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
  type JsonFile = {
    [icons: string]: {
      url: string;
      icon: string;
      name: string;
    }[];
  } & {
    variables: {
      name: string;
      value: string;
    }[];
  };

  const getLocalConfig = async (): Promise<Config> => {
    try {
      console.log("Getting config from", BaseDirectory.AppConfig);
      const contents = toJson<Config>(
        await readTextFile("config.yaml", {
          dir: BaseDirectory.AppConfig,
        })
      );
      config.gistId = contents.gistId || "";
      config.token = contents.token || "";

      return contents;
    } catch (e) {
      await createDir("", { dir: BaseDirectory.AppConfig, recursive: true });
      await writeTextFile("config.yaml", "", {
        dir: BaseDirectory.AppConfig,
      });
      // return getLocalConfig();
    }
  };

  async function setLocalConfig(partialConfig: Partial<Config>) {
    try {
      console.log("Getting config");
      const finalConfig = {
        ...config,
        ...partialConfig,
      };
      const yaml = toYaml(finalConfig);
      await writeTextFile("config.yaml", yaml, {
        dir: BaseDirectory.AppConfig,
      });
      getLocalConfig();
    } catch (e) {
      await createDir("", { dir: BaseDirectory.AppConfig, recursive: true });
      return setLocalConfig(partialConfig);
    }
  }
  let json: { content: Omit<JsonFile, "variables">; meta?: Config } = {
    content: {},
  };

  $: {
    (async function () {
      if (json.meta === config) return;
      if (config.gistId && config.token) {
        const res = await fetch(
          "https://api.github.com/gists/" +
            config.gistId +
            "?time=" +
            new Date(),
          {
            headers: {
              "X-GitHub-Api-Version": "2022-11-28",
              Authorization: "Bearer " + config.token,
              Accept: "application/vnd.github+json",
            },
          }
        ).then((res) => res.json());
        const files = res.files;
        console.log("files", files);
        for (let file in files) {
          const jsonFile = toJson<JsonFile>(files[file].content);
          console.log("json", jsonFile);
          const variables = jsonFile.variables;

          json.meta = config;
          Object.keys(jsonFile).forEach((category) => {
            if (category === "variables") return;
            json.content[category] = jsonFile[category].map((icon) => ({
              ...icon,
              url: variables.reduce((url, variable) => {
                return url.replace("$" + variable.name, variable.value);
              }, icon.url),
              icon: variables.reduce((icon, variable) => {
                return icon.replace("$" + variable.name, variable.value);
              }, icon.icon),
            }));
          });
        }
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
          on:click={() => openIcon(icon.url)}
          on:keyup={(e) => e.key === "Enter" && openIcon(icon.url)}
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
