import { Database } from "lucide-react";
import { ConnectionForm } from "./components/ConnectionForm";

function App() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="container mx-auto p-8">
        <div className="flex items-center gap-3 mb-8">
          <Database className="w-8 h-8 text-blue-600 dark:text-blue-400" />
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
            DataForge
          </h1>
        </div>

        <ConnectionForm />
      </div>
    </div>
  );
}

export default App;
