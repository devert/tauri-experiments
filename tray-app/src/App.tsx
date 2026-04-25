import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

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

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        <img src={reactLogo} className="logo react" alt="React logo" />
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

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
    </main>
  );
}

export default App;
