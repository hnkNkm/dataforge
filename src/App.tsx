import { useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { ConnectionForm } from "./components/ConnectionForm";
import { Database, FileText, History, Play } from "lucide-react";
import { Button } from "./components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import { useConnectionStore } from "./stores/connectionStore";
import { ConnectionProfile } from "./types/profile";

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [showConnectionForm, setShowConnectionForm] = useState(false);
  const [editingProfile, setEditingProfile] = useState<ConnectionProfile | null>(null);
  const { currentProfile } = useConnectionStore();

  const handleNewConnection = () => {
    setEditingProfile(null);
    setShowConnectionForm(true);
  };

  const handleEditConnection = (profile: ConnectionProfile) => {
    setEditingProfile(profile);
    setShowConnectionForm(true);
  };

  const handleCloseForm = () => {
    setShowConnectionForm(false);
    setEditingProfile(null);
  };

  return (
    <div className="h-screen flex bg-background">
      {/* サイドバー */}
      <Sidebar
        isOpen={sidebarOpen}
        onToggle={() => setSidebarOpen(!sidebarOpen)}
        onNewConnection={handleNewConnection}
        onEditConnection={handleEditConnection}
      />

      {/* メインコンテンツ */}
      <div className="flex-1 flex flex-col">
        {/* ヘッダー */}
        <header className="bg-background border-b px-4 py-2 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <Database className="w-5 h-5 text-primary" />
              <span className="font-semibold">DataForge</span>
            </div>
            {currentProfile && (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <span>{currentProfile.name}</span>
                <span>•</span>
                <span>{currentProfile.host}:{currentProfile.port}/{currentProfile.database}</span>
              </div>
            )}
          </div>
        </header>

        {/* コンテンツエリア */}
        <div className="flex-1 overflow-hidden">
          {showConnectionForm ? (
            <div className="p-6">
              <button
                onClick={handleCloseForm}
                className="mb-4 text-primary hover:underline text-sm"
              >
                ← 戻る
              </button>
              <ConnectionForm
                editingProfile={editingProfile}
                onSaveSuccess={handleCloseForm}
                isEditMode={!!editingProfile}
              />
            </div>
          ) : currentProfile ? (
            <div className="h-full flex flex-col">
              <Tabs defaultValue="query" className="flex-1 flex flex-col">
                <div className="border-b px-4">
                  <TabsList className="h-10 bg-transparent">
                    <TabsTrigger value="query" className="gap-2">
                      <FileText className="w-4 h-4" />
                      クエリ
                    </TabsTrigger>
                    <TabsTrigger value="history" className="gap-2">
                      <History className="w-4 h-4" />
                      履歴
                    </TabsTrigger>
                  </TabsList>
                </div>
                <TabsContent value="query" className="flex-1 m-0 p-4">
                  <div className="h-full flex flex-col gap-4">
                    <div className="flex-1 bg-muted/20 rounded-md p-4 border">
                      <div className="flex items-center justify-between mb-4">
                        <div className="text-sm text-muted-foreground">
                          SQLエディター
                        </div>
                        <Button size="sm" className="gap-2">
                          <Play className="w-4 h-4" />
                          実行
                        </Button>
                      </div>
                      <textarea
                        className="w-full h-[calc(100%-3rem)] bg-transparent resize-none outline-none font-mono text-sm"
                        placeholder="SELECT * FROM ..."
                      />
                    </div>
                    <div className="h-1/3 bg-background border rounded-md">
                      <div className="border-b px-4 py-2">
                        <h3 className="text-sm font-semibold">結果</h3>
                      </div>
                      <div className="p-4">
                        <div className="text-sm text-muted-foreground">
                          クエリを実行すると結果がここに表示されます
                        </div>
                      </div>
                    </div>
                  </div>
                </TabsContent>
                <TabsContent value="history" className="flex-1 m-0 p-4">
                  <div className="text-sm text-muted-foreground">
                    クエリ履歴
                  </div>
                </TabsContent>
              </Tabs>
            </div>
          ) : (
            <div className="h-full flex items-center justify-center">
              <div className="text-center">
                <Database className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                <p className="text-muted-foreground mb-4">
                  データベースに接続してください
                </p>
                <Button onClick={handleNewConnection}>
                  接続を選択
                </Button>
              </div>
            </div>
          )}
        </div>

        {/* ステータスバー */}
        <footer className="bg-background border-t px-4 py-1 flex items-center justify-between text-xs text-muted-foreground">
          <div className="flex items-center gap-4">
            <span>{currentProfile ? '接続中' : '未接続'}</span>
          </div>
          <div className="flex items-center gap-4">
            <span>UTF-8</span>
          </div>
        </footer>
      </div>
    </div>
  );
}

export default App;
