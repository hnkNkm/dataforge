import { useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { ConnectionForm } from "./components/ConnectionForm";
import { DatabaseExplorerEnhanced } from "./components/DatabaseExplorerEnhanced";
import { TableView } from "./components/TableView";
import { MonacoQueryEditor } from "./components/MonacoQueryEditor";
import { Database, FileText, History, Play, FolderTree, Plus, X, Settings } from "lucide-react";
import { Button } from "./components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import { Toaster } from "./components/ui/sonner";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "./components/ui/resizable";
import { useConnectionStore } from "./stores/connectionStore";
import { useTabStore } from "./stores/tabStore";
import { ConnectionProfile } from "./types/profile";
import { cn } from "./lib/utils";

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [showConnectionForm, setShowConnectionForm] = useState(false);
  const [editingProfile, setEditingProfile] = useState<ConnectionProfile | null>(null);
  const { currentProfile } = useConnectionStore();
  const { tabs, activeTabId, removeTab, setActiveTab, openQueryTab, updateTab } = useTabStore();

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
            <ResizablePanelGroup direction="horizontal">
              {/* データベースエクスプローラー */}
              <ResizablePanel defaultSize={25} minSize={15} maxSize={40}>
                <Tabs defaultValue="explorer" className="h-full flex flex-col">
                  <div className="border-b px-2">
                    <TabsList className="h-10 bg-transparent">
                      <TabsTrigger value="explorer" className="gap-2">
                        <FolderTree className="w-4 h-4" />
                        エクスプローラー
                      </TabsTrigger>
                    </TabsList>
                  </div>
                  <TabsContent value="explorer" className="flex-1 m-0">
                    <DatabaseExplorerEnhanced />
                  </TabsContent>
                </Tabs>
              </ResizablePanel>

              <ResizableHandle />

              {/* メインコンテンツエリア */}
              <ResizablePanel defaultSize={75}>
                <div className="h-full flex flex-col">
                  {/* タブヘッダー */}
                  <div className="border-b flex items-center">
                    <div className="flex-1 flex items-center overflow-x-auto">
                      {tabs.map((tab) => (
                        <div
                          key={tab.id}
                          className={cn(
                            "flex items-center gap-2 px-3 py-2 border-r cursor-pointer hover:bg-accent/50 min-w-[120px]",
                            activeTabId === tab.id && "bg-accent"
                          )}
                          onClick={() => setActiveTab(tab.id)}
                        >
                          {tab.type === 'query' ? (
                            <FileText className="w-3 h-3" />
                          ) : (
                            <Database className="w-3 h-3" />
                          )}
                          <span className="text-sm truncate flex-1">{tab.title}</span>
                          {tabs.length > 1 && (
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                removeTab(tab.id);
                              }}
                              className="hover:bg-destructive/20 rounded p-0.5"
                            >
                              <X className="w-3 h-3" />
                            </button>
                          )}
                        </div>
                      ))}
                      <Button
                        variant="ghost"
                        size="sm"
                        className="ml-2"
                        onClick={() => openQueryTab()}
                      >
                        <Plus className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>

                  {/* タブコンテンツ */}
                  <div className="flex-1 overflow-hidden">
                    {tabs.map((tab) => (
                      <div
                        key={tab.id}
                        className={cn(
                          "h-full",
                          activeTabId === tab.id ? "block" : "hidden"
                        )}
                      >
                        {tab.type === 'query' ? (
                          <MonacoQueryEditor
                            initialContent={tab.content || 'SELECT * FROM '}
                            onContentChange={(content) => updateTab(tab.id, { content })}
                          />
                        ) : (
                          <TableView tableName={tab.tableName || ''} />
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </ResizablePanel>
            </ResizablePanelGroup>
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
      <Toaster />
    </div>
  );
}

export default App;
