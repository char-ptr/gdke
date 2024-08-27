import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Input } from "./components/ui/input";
import { open } from "@tauri-apps/plugin-dialog";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  function get_file() {
    open({
      title: "File picker",
      filters: [{ name: "Applications", extensions: ["exe"] }],
      directory: false,
    });
  }

  return (
    <div className="container dark:bg-black dark:text-white">
      <h1 className="text-2xl tracking-tight">👋 Welcome to Gdke</h1>
      <p className="text-neutral-400 text-lg tracking-wide">
        To get started drop your program below:
      </p>

      <div className="flex items-center justify-center w-full">
        <label
          htmlFor="dropzone-file"
          className="flex flex-col items-center justify-center w-full h-64 border-2 border-gray-300 border-dashed rounded-lg cursor-pointer bg-neutral-50-50 dark:hover:bg-neutral-800 dark:bg-neutral-900 hover:bg-neutral-100 dark:border-neutral-600 dark:hover:border-neutral-500"
        >
          <div className="flex flex-col items-center justify-center pt-5 pb-6">
            <svg
              className="w-8 h-8 mb-4 text-gray-500 dark:text-gray-400"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 20 16"
            >
              <path
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 13h3a3 3 0 0 0 0-6h-.025A5.56 5.56 0 0 0 16 6.5 5.5 5.5 0 0 0 5.207 5.021C5.137 5.017 5.071 5 5 5a4 4 0 0 0 0 8h2.167M10 15V6m0 0L8 8m2-2 2 2"
              />
            </svg>
            <p className="mb-2 text-sm text-gray-500 dark:text-gray-400">
              <span className="font-semibold">Click to upload</span> or drag and
              drop
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              EXE, Application files
            </p>
          </div>
          <button
            onClick={() => get_file()}
            id="dropzone-file"
            type="button"
            className="hidden"
          />
        </label>
      </div>
    </div>
  );
}

export default App;
