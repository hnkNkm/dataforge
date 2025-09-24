import { useState } from "react";
import { Database } from "lucide-react";
import { ConnectionForm } from "./components/ConnectionForm";
import { ConnectionProfiles } from "./components/ConnectionProfiles";
import { ConnectionStatus } from "./components/ConnectionStatus";
import { ConnectionProfile } from "./types/profile";

type ViewMode = "profiles" | "new" | "edit";

function App() {
  const [viewMode, setViewMode] = useState<ViewMode>("profiles");
  const [editingProfile, setEditingProfile] = useState<ConnectionProfile | null>(null);

  const handleCreateNew = () => {
    setEditingProfile(null);
    setViewMode("new");
  };

  const handleEditProfile = (profile: ConnectionProfile) => {
    setEditingProfile(profile);
    setViewMode("edit");
  };

  const handleBackToProfiles = () => {
    setViewMode("profiles");
    setEditingProfile(null);
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex flex-col">
      {/* Header */}
      <div className="bg-white dark:bg-gray-800 shadow-sm">
        <div className="container mx-auto px-8 py-4">
          <div className="flex items-center gap-3">
            <Database className="w-8 h-8 text-blue-600 dark:text-blue-400" />
            <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              DataForge
            </h1>
          </div>
        </div>
      </div>

      {/* Connection Status Bar */}
      <ConnectionStatus />

      {/* Main Content */}
      <div className="flex-1 container mx-auto px-8 py-6">
        {viewMode === "profiles" ? (
          <ConnectionProfiles
            onCreateNew={handleCreateNew}
            onEditProfile={handleEditProfile}
          />
        ) : (
          <div>
            <button
              onClick={handleBackToProfiles}
              className="mb-4 text-blue-600 dark:text-blue-400 hover:underline text-sm"
            >
              ← プロファイル一覧に戻る
            </button>
            <ConnectionForm
              editingProfile={editingProfile}
              onSaveSuccess={handleBackToProfiles}
              isEditMode={viewMode === "edit"}
            />
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
