import React from 'react';
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Database, FileText, History, Settings } from "lucide-react";

interface MainLayoutProps {
  children?: React.ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  return (
    <div className="h-screen flex flex-col">
      {/* ヘッダー */}
      <header className="bg-background border-b px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5 text-primary" />
            <span className="font-semibold">DataForge</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button className="p-2 hover:bg-accent rounded-md">
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </header>

      {/* メインコンテンツエリア */}
      <div className="flex-1 overflow-hidden">
        <ResizablePanelGroup
          direction="horizontal"
          className="h-full"
        >
          {/* 左サイドバー - データベースエクスプローラー */}
          <ResizablePanel
            defaultSize={20}
            minSize={15}
            maxSize={30}
            className="bg-sidebar"
          >
            <div className="h-full flex flex-col">
              <div className="p-3 border-b">
                <h3 className="text-sm font-semibold text-sidebar-foreground">エクスプローラー</h3>
              </div>
              <div className="flex-1 overflow-auto p-3">
                {/* ここにデータベースツリーが入る */}
                <div className="text-sm text-muted-foreground">
                  接続を選択してください
                </div>
              </div>
            </div>
          </ResizablePanel>

          <ResizableHandle className="w-1 bg-border" />

          {/* 中央パネル - エディター/コンテンツエリア */}
          <ResizablePanel
            defaultSize={50}
            minSize={30}
          >
            <ResizablePanelGroup direction="vertical">
              {/* 上部 - SQLエディター/メインコンテンツ */}
              <ResizablePanel
                defaultSize={60}
                minSize={30}
              >
                <div className="h-full flex flex-col">
                  <Tabs defaultValue="query" className="flex-1 flex flex-col">
                    <div className="border-b px-2">
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
                    <TabsContent value="query" className="flex-1 m-0">
                      <div className="h-full p-4">
                        {/* ここにSQLエディターが入る */}
                        <div className="h-full bg-muted/20 rounded-md p-4">
                          <div className="text-sm text-muted-foreground">
                            SQLエディターエリア
                          </div>
                        </div>
                      </div>
                    </TabsContent>
                    <TabsContent value="history" className="flex-1 m-0">
                      <div className="h-full p-4">
                        <div className="text-sm text-muted-foreground">
                          クエリ履歴
                        </div>
                      </div>
                    </TabsContent>
                  </Tabs>
                </div>
              </ResizablePanel>

              <ResizableHandle className="h-1 bg-border" />

              {/* 下部 - 結果表示エリア */}
              <ResizablePanel
                defaultSize={40}
                minSize={20}
              >
                <div className="h-full flex flex-col">
                  <div className="border-b px-4 py-2">
                    <h3 className="text-sm font-semibold">結果</h3>
                  </div>
                  <div className="flex-1 overflow-auto p-4">
                    {/* ここに結果テーブルが入る */}
                    <div className="text-sm text-muted-foreground">
                      クエリを実行すると結果がここに表示されます
                    </div>
                  </div>
                </div>
              </ResizablePanel>
            </ResizablePanelGroup>
          </ResizablePanel>

          <ResizableHandle className="w-1 bg-border" />

          {/* 右サイドバー - プロパティ/詳細 */}
          <ResizablePanel
            defaultSize={30}
            minSize={20}
            maxSize={40}
            collapsible={true}
          >
            <div className="h-full flex flex-col">
              <div className="p-3 border-b">
                <h3 className="text-sm font-semibold">詳細</h3>
              </div>
              <div className="flex-1 overflow-auto p-3">
                {children || (
                  <div className="text-sm text-muted-foreground">
                    テーブルやカラムを選択すると詳細が表示されます
                  </div>
                )}
              </div>
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>

      {/* ステータスバー */}
      <footer className="bg-background border-t px-4 py-1 flex items-center justify-between text-xs text-muted-foreground">
        <div className="flex items-center gap-4">
          <span>準備完了</span>
        </div>
        <div className="flex items-center gap-4">
          <span>UTF-8</span>
        </div>
      </footer>
    </div>
  );
}