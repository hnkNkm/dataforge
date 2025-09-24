import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Database } from "lucide-react";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="container mx-auto p-8">
        <div className="flex items-center gap-3 mb-8">
          <Database className="w-8 h-8 text-blue-600 dark:text-blue-400" />
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            DataForge
          </h1>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
          <h2 className="text-xl font-semibold mb-4 text-gray-800 dark:text-gray-200">
            Database Client
          </h2>

          <form
            className="flex gap-4"
            onSubmit={(e) => {
              e.preventDefault();
              greet();
            }}
          >
            <input
              className="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
              onChange={(e) => setName(e.currentTarget.value)}
              placeholder="Enter a name..."
            />
            <button
              type="submit"
              className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors"
            >
              Greet
            </button>
          </form>

          {greetMsg && (
            <p className="mt-4 p-4 bg-gray-100 dark:bg-gray-700 rounded-md text-gray-700 dark:text-gray-300">
              {greetMsg}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
