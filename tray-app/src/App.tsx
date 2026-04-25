import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [username, setUsername] = useState("");
  const [theme, setTheme] = useState("light");
  const [settingsMsg, setSettingsMsg] = useState("");
  const [processMsg, setProcessMsg] = useState("");

  useEffect(() => {
    const handler = () => {
      if (document.visibilityState === "hidden") {
        setGreetMsg("");
        setName("");
      }
    };
    document.addEventListener("visibilitychange", handler);
    return () => document.removeEventListener("visibilitychange", handler);
  }, []);

  async function loadSettings() {
    try {
      const s = await invoke<{ username: string; theme: string }>(
        "load_settings",
      );
      setUsername(s.username);
      setTheme(s.theme);
      setSettingsMsg("Settings loaded.");
    } catch (e) {
      setSettingsMsg(`Error loading: ${e}`);
    }
  }

  async function saveSettings() {
    try {
      await invoke("save_settings", {
        settings: { username, theme },
      });
      setSettingsMsg("Settings saved.");
    } catch (e) {
      setSettingsMsg(`Error saving: ${e}`);
    }
  }

  async function spawnProcess() {
    try {
      const msg = await invoke<string>("spawn_process");
      setProcessMsg(msg);
    } catch (e) {
      setProcessMsg(`Error: ${e}`);
    }
  }

  async function checkStatus() {
    try {
      const msg = await invoke<string>("process_status");
      setProcessMsg(msg);
    } catch (e) {
      setProcessMsg(`Error: ${e}`);
    }
  }

  async function killProcess() {
    try {
      const msg = await invoke<string>("kill_process");
      setProcessMsg(msg);
    } catch (e) {
      setProcessMsg(`Error: ${e}`);
    }
  }

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          value={name}
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <hr />
      <h2>Settings</h2>

      <div className="row">
        <label>
          Username:{" "}
          <input
            value={username}
            onChange={(e) => setUsername(e.currentTarget.value)}
            placeholder="Enter username..."
          />
        </label>
      </div>

      <div className="row">
        <label>
          Theme:{" "}
          <select
            value={theme}
            onChange={(e) => setTheme(e.currentTarget.value)}
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>
      </div>

      <div className="row">
        <button onClick={loadSettings}>Load Settings</button>
        <button onClick={saveSettings}>Save Settings</button>
      </div>

      <p>{settingsMsg}</p>

      <hr />
      <h2>Process Manager</h2>
      <div className="row">
        <button onClick={spawnProcess}>Spawn ping</button>
        <button onClick={checkStatus}>Check Status</button>
        <button onClick={killProcess}>Kill</button>
      </div>
      <p>{processMsg}</p>
    </main>
  );
}

export default App;
