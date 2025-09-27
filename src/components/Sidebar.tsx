import React, { useState } from 'react';
import {
  ChevronRight,
  ChevronDown,
  Database,
  Server,
  HardDrive,
  Folder,
  Table,
  Plus,
  Settings,
  FolderOpen,
  Menu,
  X,
  Play,
  Edit2,
  Trash2,
  Loader2,
  AlertTriangle,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { useConnectionStore } from '@/stores/connectionStore';
import { useProfileStore } from '@/stores/profileStore';
import { ConnectionProfile } from '@/types/profile';
import { cn } from '@/lib/utils';
import { toast } from 'sonner';

interface SidebarProps {
  isOpen: boolean;
  onToggle: () => void;
  onNewConnection: () => void;
  onEditConnection: (profile: ConnectionProfile) => void;
}

interface ProjectGroup {
  name: string;
  profiles: ConnectionProfile[];
}

export function Sidebar({ isOpen, onToggle, onNewConnection, onEditConnection }: SidebarProps) {
  const { profiles, deleteProfile } = useProfileStore();
  const { connectWithProfile, currentProfile, disconnect, isConnecting } = useConnectionStore();
  const [expandedProjects, setExpandedProjects] = useState<Set<string>>(new Set(['default']));
  const [expandedConnections, setExpandedConnections] = useState<Set<string>>(new Set());
  const [connectingId, setConnectingId] = useState<string | null>(null);
  const [deletingProfile, setDeletingProfile] = useState<ConnectionProfile | null>(null);

  // プロジェクトごとにグループ化
  const groupedProfiles = profiles.reduce<Record<string, ConnectionProfile[]>>((groups, profile) => {
    const project = profile.project || 'デフォルト';
    if (!groups[project]) {
      groups[project] = [];
    }
    groups[project].push(profile);
    return groups;
  }, {});

  const toggleProject = (project: string) => {
    const newExpanded = new Set(expandedProjects);
    if (newExpanded.has(project)) {
      newExpanded.delete(project);
    } else {
      newExpanded.add(project);
    }
    setExpandedProjects(newExpanded);
  };

  const toggleConnection = (connectionId: string) => {
    const newExpanded = new Set(expandedConnections);
    if (newExpanded.has(connectionId)) {
      newExpanded.delete(connectionId);
    } else {
      newExpanded.add(connectionId);
    }
    setExpandedConnections(newExpanded);
  };

  const getDatabaseIcon = (type: string) => {
    switch (type) {
      case 'postgresql':
        return <Database className="w-4 h-4 text-blue-500" />;
      case 'mysql':
        return <Server className="w-4 h-4 text-orange-500" />;
      case 'sqlite':
        return <HardDrive className="w-4 h-4 text-green-500" />;
      default:
        return <Database className="w-4 h-4" />;
    }
  };

  const handleConnect = async (profile: ConnectionProfile) => {
    setConnectingId(profile.id);
    try {
      await connectWithProfile(profile.id);
      toast.success('接続成功', {
        description: `${profile.name}に接続しました`,
      });
    } catch (error) {
      console.error('Failed to connect:', error);
      toast.error('接続失敗', {
        description: error instanceof Error ? error.message : '接続に失敗しました',
      });
    } finally {
      setConnectingId(null);
    }
  };

  const handleDelete = async (profile: ConnectionProfile) => {
    try {
      await deleteProfile(profile.id);
      toast.success('削除完了', {
        description: `${profile.name}を削除しました`,
      });
      setDeletingProfile(null);
    } catch (error) {
      console.error('Failed to delete:', error);
      toast.error('削除失敗', {
        description: '削除に失敗しました',
      });
    }
  };

  const handleDisconnect = async () => {
    try {
      await disconnect();
      toast.success('切断しました');
    } catch (error) {
      console.error('Failed to disconnect:', error);
      toast.error('切断に失敗しました');
    }
  };

  return (
    <div
      className={cn(
        "flex flex-col h-full bg-sidebar border-r transition-all duration-300",
        isOpen ? "w-64" : "w-12"
      )}
    >
      {/* サイドバーヘッダー */}
      <div className="p-2 border-b flex items-center justify-between">
        <div className={cn("flex items-center gap-2", !isOpen && "hidden")}>
          <Database className="w-5 h-5 text-primary" />
          <span className="font-semibold text-sm">接続管理</span>
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={onToggle}
          className="h-8 w-8"
        >
          {isOpen ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
        </Button>
      </div>

      {/* サイドバーコンテンツ */}
      {isOpen ? (
        <ScrollArea className="flex-1">
          <div className="p-2 space-y-2">
            {/* 新規接続ボタン */}
            <Button
              onClick={onNewConnection}
              variant="outline"
              size="sm"
              className="w-full justify-start gap-2"
            >
              <Plus className="w-4 h-4" />
              新規接続
            </Button>

            <Separator className="my-2" />

            {/* プロジェクトごとの接続一覧 */}
            {Object.entries(groupedProfiles).map(([project, projectProfiles]) => (
              <Collapsible
                key={project}
                open={expandedProjects.has(project)}
                onOpenChange={() => toggleProject(project)}
              >
                <CollapsibleTrigger className="flex items-center gap-1 w-full p-1 hover:bg-accent rounded text-sm">
                  {expandedProjects.has(project) ? (
                    <ChevronDown className="w-3 h-3" />
                  ) : (
                    <ChevronRight className="w-3 h-3" />
                  )}
                  {expandedProjects.has(project) ? (
                    <FolderOpen className="w-4 h-4" />
                  ) : (
                    <Folder className="w-4 h-4" />
                  )}
                  <span className="flex-1 text-left">{project}</span>
                  <span className="text-xs text-muted-foreground">
                    {projectProfiles.length}
                  </span>
                </CollapsibleTrigger>
                <CollapsibleContent className="ml-2">
                  {projectProfiles.map((profile) => {
                    const isConnected = currentProfile?.id === profile.id;
                    const isExpanded = expandedConnections.has(profile.id);

                    return (
                      <div key={profile.id} className="mt-1">
                        <div
                          className={cn(
                            "flex items-center gap-1 p-1 rounded text-sm cursor-pointer",
                            isConnected ? "bg-accent" : "hover:bg-accent/50"
                          )}
                          onDoubleClick={() => !isConnected && handleConnect(profile)}
                        >
                          <button
                            onClick={() => toggleConnection(profile.id)}
                            className="p-0.5"
                          >
                            {isExpanded ? (
                              <ChevronDown className="w-3 h-3" />
                            ) : (
                              <ChevronRight className="w-3 h-3" />
                            )}
                          </button>
                          {getDatabaseIcon(profile.database_type)}
                          <span className="flex-1 truncate">{profile.name}</span>
                          <div className="flex items-center gap-0.5">
                            {!isConnected && (
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6"
                                onClick={() => handleConnect(profile)}
                                disabled={connectingId === profile.id}
                              >
                                {connectingId === profile.id ? (
                                  <Loader2 className="w-3 h-3 animate-spin" />
                                ) : (
                                  <Play className="w-3 h-3" />
                                )}
                              </Button>
                            )}
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-6 w-6"
                              onClick={() => onEditConnection(profile)}
                              disabled={isConnected}
                            >
                              <Edit2 className="w-3 h-3" />
                            </Button>
                            {!isConnected && (
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6 hover:text-destructive"
                                onClick={() => setDeletingProfile(profile)}
                              >
                                <Trash2 className="w-3 h-3" />
                              </Button>
                            )}
                          </div>
                        </div>
                        {isExpanded && (
                          <div className="ml-7 text-xs text-muted-foreground space-y-0.5 mt-1">
                            <div>{profile.host}:{profile.port}</div>
                            <div>DB: {profile.database}</div>
                            <div>User: {profile.username}</div>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </CollapsibleContent>
              </Collapsible>
            ))}

            {profiles.length === 0 && (
              <div className="text-center py-8 text-sm text-muted-foreground">
                <Database className="w-8 h-8 mx-auto mb-2 opacity-50" />
                <p>接続がありません</p>
              </div>
            )}
          </div>
        </ScrollArea>
      ) : (
        /* 折りたたみ時のアイコン表示 */
        <div className="flex-1 flex flex-col items-center py-2 space-y-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={onNewConnection}
            className="h-8 w-8"
            title="新規接続"
          >
            <Plus className="w-4 h-4" />
          </Button>
          {currentProfile && (
            <div className="flex flex-col items-center">
              {getDatabaseIcon(currentProfile.database_type)}
            </div>
          )}
        </div>
      )}

      {/* サイドバーフッター */}
      <div className="p-2 border-t">
        {currentProfile && isOpen && (
          <div className="text-xs space-y-1 mb-2">
            <div className="text-muted-foreground">現在の接続:</div>
            <div className="font-medium truncate">{currentProfile.name}</div>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDisconnect}
              className="w-full mt-1"
            >
              切断
            </Button>
          </div>
        )}
        <Button
          variant="ghost"
          size={isOpen ? "sm" : "icon"}
          className={cn("w-full", !isOpen && "h-8 w-8")}
        >
          <Settings className="w-4 h-4" />
          {isOpen && <span className="ml-2">設定</span>}
        </Button>
      </div>

      {/* 削除確認ダイアログ */}
      <AlertDialog open={!!deletingProfile} onOpenChange={() => setDeletingProfile(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <div className="flex items-center gap-2">
                <AlertTriangle className="w-5 h-5 text-destructive" />
                接続プロファイルの削除
              </div>
            </AlertDialogTitle>
            <AlertDialogDescription>
              「{deletingProfile?.name}」を削除してもよろしいですか？
              この操作は取り消すことができません。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>キャンセル</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deletingProfile && handleDelete(deletingProfile)}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              削除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}