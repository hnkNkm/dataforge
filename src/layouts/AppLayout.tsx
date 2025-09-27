import React, { useState } from 'react';
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { 
  Database, 
  FileText, 
  History, 
  Settings,
  Plus,
  ChevronRight,
  ChevronDown,
  Server,
  HardDrive,
  Folder,
  Table,
  Play
} from "lucide-react";
import { ConnectionProfiles } from "@/components/ConnectionProfiles";
import { useConnectionStore } from "@/stores/connectionStore";
import { Button } from "@/components/ui/button";

interface AppLayoutProps {
  children?: React.ReactNode;
}

// データベース接続ツリーアイテム
interface TreeNode {
  id: string;
  label: string;
  type: 'connection' | 'database' | 'schema' | 'table' | 'view';
  icon?: React.ReactNode;
  children?: TreeNode[];
  expanded?: boolean;
}

export function AppLayout({ children }: AppLayoutProps) {
  const { currentConnection, disconnect } = useConnectionStore();
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [showProfiles, setShowProfiles] = useState(!currentConnection);

  // ツリーノードの展開/折りたたみ
  const toggleNode = (nodeId: string) => {
    const newExpanded = new Set(expandedNodes);
    if (newExpanded.has(nodeId)) {
      newExpanded.delete(nodeId);
    } else {
      newExpanded.add(nodeId);
    }
    setExpandedNodes(newExpanded);
  };

  // サンプルツリーデータ（実際は接続情報から生成）
  const treeData: TreeNode[] = currentConnection ? [
    {
      id: 'conn1',
      label: currentConnection.name,
      type: 'connection',
      icon: currentConnection.database_type === 'postgresql' ? 
        <Database className="w-4 h-4 text-blue-500" /> :
        currentConnection.database_type === 'mysql' ?
        <Server className="w-4 h-4 text-orange-500" /> :
        <HardDrive className="w-4 h-4 text-green-500" />,
      children: [
        {
          id: 'db1',
          label: currentConnection.database,
          type: 'database',
          icon: <Folder className="w-4 h-4" />,
          children: [
            {
              id: 'schema1',
              label: 'public',
              type: 'schema',
              icon: <Folder className="w-4 h-4" />,
              children: [
                { id: 'table1', label: 'users', type: 'table', icon: <Table className="w-4 h-4" /> },
                { id: 'table2', label: 'products', type: 'table', icon: <Table className="w-4 h-4" /> },
              ]
            }
          ]
        }
      ]
    }
  ] : [];

  // ツリーアイテムのレンダリング
  const renderTreeNode = (node: TreeNode, level: number = 0) => {
    const isExpanded = expandedNodes.has(node.id);
    const hasChildren = node.children && node.children.length > 0;

    return (
      <div key={node.id}>
        <div
          className="flex items-center gap-1 py-1 px-2 hover:bg-accent rounded cursor-pointer text-sm"
          style={{ paddingLeft: `${level * 16 + 8}px` }}
          onClick={() => hasChildren && toggleNode(node.id)}
        >
          {hasChildren && (
            <div className="w-4 h-4 flex items-center justify-center">
              {isExpanded ? 
                <ChevronDown className="w-3 h-3" /> : 
                <ChevronRight className="w-3 h-3" />
              }
            </div>
          )}
          {!hasChildren && <div className="w-4" />}
          {node.icon}
          <span className="ml-1">{node.label}</span>
        </div>
        {isExpanded && node.children && (
          <div>
            {node.children.map(child => renderTreeNode(child, level + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* ヘッダー */}
      <header className="bg-background border-b px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5 text-primary" />
            <span className="font-semibold">DataForge</span>
          </div>
          {currentConnection && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <span>{currentConnection.name}</span>
              <span>•</span>
              <span>{currentConnection.host}:{currentConnection.port}</span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          {currentConnection && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                disconnect();
                setShowProfiles(true);
              }}
            >
              切断
            </Button>
          )}
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
          {/* 左サイドバー - 接続管理とエクスプローラー */}
          <ResizablePanel
            defaultSize={25}
            minSize={20}
            maxSize={35}
            className="bg-sidebar"
          >
            <div className="h-full flex flex-col">
              <Tabs defaultValue="connections" className="flex-1 flex flex-col">
                <div className="border-b px-2">
                  <TabsList className="h-10 bg-transparent w-full justify-start">
                    <TabsTrigger value="connections" className="gap-2">
                      <Database className="w-4 h-4" />
                      接続
                    </TabsTrigger>
                    <TabsTrigger value="explorer" className="gap-2">
                      <Folder className="w-4 h-4" />
                      エクスプローラー
                    </TabsTrigger>
                  </TabsList>
                </div>
                
                <TabsContent value="connections" className="flex-1 m-0 overflow-auto">
                  <div className="p-3">
                    {showProfiles || !currentConnection ? (
                      <div className="space-y-2">
                        <ConnectionProfiles
                          onCreateNew={() => {}}
                          onEditProfile={() => {}}
                          onConnectSuccess={() => setShowProfiles(false)}
                        />
                      </div>
                    ) : (
                      <div className="space-y-2">
                        <Button 
                          onClick={() => setShowProfiles(true)}
                          variant="outline" 
                          size="sm" 
                          className="w-full justify-start gap-2"
                        >
                          <Plus className="w-4 h-4" />
                          新しい接続を追加
                        </Button>
                        <div className="mt-4">
                          <h3 className="text-xs font-semibold text-muted-foreground mb-2">アクティブな接続</h3>
                          {treeData.map(node => renderTreeNode(node))}
                        </div>
                      </div>
                    )}
                  </div>
                </TabsContent>
                
                <TabsContent value="explorer" className="flex-1 m-0 overflow-auto">
                  <div className="p-3">
                    {currentConnection ? (
                      <div className="space-y-2">
                        {treeData.map(node => renderTreeNode(node))}
                      </div>
                    ) : (
                      <div className="text-sm text-muted-foreground">
                        データベースに接続してください
                      </div>
                    )}
                  </div>
                </TabsContent>
              </Tabs>
            </div>
          </ResizablePanel>

          <ResizableHandle className="w-1 bg-border" />

          {/* 中央パネル - メインコンテンツ */}
          <ResizablePanel
            defaultSize={75}
            minSize={50}
          >
            {currentConnection ? (
              <ResizablePanelGroup direction="vertical">
                {/* 上部 - SQLエディター */}
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
                          <div className="h-full bg-muted/20 rounded-md p-4 border">
                            <div className="flex items-center justify-between mb-4">
                              <div className="text-sm text-muted-foreground">
                                SQLエディターエリア
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
                      <div className="text-sm text-muted-foreground">
                        クエリを実行すると結果がここに表示されます
                      </div>
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
                  <Button onClick={() => setShowProfiles(true)}>
                    接続を選択
                  </Button>
                </div>
              </div>
            )}
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>

      {/* ステータスバー */}
      <footer className="bg-background border-t px-4 py-1 flex items-center justify-between text-xs text-muted-foreground">
        <div className="flex items-center gap-4">
          <span>{currentConnection ? '接続中' : '未接続'}</span>
        </div>
        <div className="flex items-center gap-4">
          <span>UTF-8</span>
        </div>
      </footer>
    </div>
  );
}