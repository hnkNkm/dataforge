import React, { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  ChevronRight,
  ChevronDown,
  Table,
  Database,
  Columns,
  Key,
  Loader2,
  RefreshCw,
  AlertCircle,
  Zap,
  Play,
  Copy,
  FileText,
  Info,
  MoreVertical,
  Hash,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from '@/components/ui/dropdown-menu';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useConnectionStore } from '@/stores/connectionStore';
import { useTabStore } from '@/stores/tabStore';
import { cn } from '@/lib/utils';
import { toast } from 'sonner';

interface TableInfo {
  name: string;
  type: 'table' | 'view';
  schema?: string;
}

interface ColumnInfo {
  name: string;
  type: string;
  nullable: boolean;
  primary_key?: boolean;
  default?: string;
}

interface IndexInfo {
  index_name: string;
  is_primary?: boolean;
  is_unique?: boolean;
  definition?: string;
  size?: string;
}

interface MetadataCache {
  tables?: TableInfo[];
  columns?: Record<string, ColumnInfo[]>;
  indexes?: Record<string, IndexInfo[]>;
  timestamp: number;
}

const CACHE_DURATION = 5 * 60 * 1000; // 5分間キャッシュ

export function DatabaseExplorerEnhanced() {
  const { currentProfile } = useConnectionStore();
  const { openTableTab, openQueryTab } = useTabStore();
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());
  const [tableColumns, setTableColumns] = useState<Record<string, ColumnInfo[]>>({});
  const [tableIndexes, setTableIndexes] = useState<Record<string, IndexInfo[]>>({});
  const [showIndexes, setShowIndexes] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [metadataCache, setMetadataCache] = useState<MetadataCache | null>(null);

  // キャッシュからデータを取得
  const loadFromCache = useCallback(() => {
    if (metadataCache && Date.now() - metadataCache.timestamp < CACHE_DURATION) {
      if (metadataCache.tables) setTables(metadataCache.tables);
      if (metadataCache.columns) setTableColumns(metadataCache.columns);
      if (metadataCache.indexes) setTableIndexes(metadataCache.indexes);
      return true;
    }
    return false;
  }, [metadataCache]);

  // キャッシュを更新
  const updateCache = useCallback((
    newTables?: TableInfo[],
    newColumns?: Record<string, ColumnInfo[]>,
    newIndexes?: Record<string, IndexInfo[]>
  ) => {
    setMetadataCache(prev => ({
      tables: newTables || prev?.tables || [],
      columns: { ...(prev?.columns || {}), ...(newColumns || {}) },
      indexes: { ...(prev?.indexes || {}), ...(newIndexes || {}) },
      timestamp: Date.now()
    }));
  }, []);

  // テーブル一覧を取得
  const fetchTables = async (useCache: boolean = true) => {
    if (!currentProfile) return;

    // キャッシュを使用する場合
    if (useCache && loadFromCache()) {
      toast.info('キャッシュからデータを読み込みました');
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const result = await invoke<any>('list_database_tables');
      
      if (Array.isArray(result)) {
        const processedTables = result.map((table: any) => ({
          name: table.name,
          type: table.table_type === 'TABLE' ? 'table' : 'view',
          schema: table.schema
        }));
        setTables(processedTables);
        updateCache(processedTables);
        
        if (result.length > 0) {
          toast.success(`${result.length}個のテーブルを取得しました`);
        } else {
          toast.info('テーブルが見つかりませんでした');
        }
      } else {
        setTables([]);
      }
    } catch (err) {
      console.error('Failed to fetch tables:', err);
      setError(err instanceof Error ? err.message : 'テーブル一覧の取得に失敗しました');
      toast.error('テーブル一覧の取得に失敗しました');
    } finally {
      setLoading(false);
    }
  };

  // テーブルのカラム情報を取得
  const fetchTableColumns = async (tableName: string) => {
    try {
      const query = currentProfile?.database_type === 'postgresql'
        ? `SELECT
            column_name,
            data_type,
            is_nullable,
            column_default
          FROM information_schema.columns
          WHERE table_name = '${tableName}'
          ORDER BY ordinal_position`
        : currentProfile?.database_type === 'mysql'
        ? `SELECT 
            COLUMN_NAME as column_name,
            COLUMN_TYPE as data_type,
            IS_NULLABLE as is_nullable,
            COLUMN_DEFAULT as column_default
          FROM information_schema.COLUMNS
          WHERE TABLE_NAME = '${tableName}'
          ORDER BY ORDINAL_POSITION`
        : `PRAGMA table_info(${tableName})`;

      const result = await invoke<any>('execute_query', { query });

      if (result && result.rows) {
        const columns: ColumnInfo[] = result.rows.map((row: any) => ({
          name: row.column_name || row.name,
          type: row.data_type || row.type,
          nullable: row.is_nullable === 'YES' || row.notnull === 0,
          primary_key: row.pk === 1,
          default: row.column_default || row.dflt_value
        }));
        
        setTableColumns(prev => ({ ...prev, [tableName]: columns }));
        updateCache(undefined, { [tableName]: columns });
      }
    } catch (err) {
      console.error('Failed to fetch columns:', err);
      toast.error(`${tableName}のカラム情報の取得に失敗しました`);
    }
  };

  // インデックス情報を取得
  const fetchTableIndexes = async (tableName: string) => {
    try {
      const result = await invoke<any>('get_table_indexes', { tableName });
      if (result && result.rows) {
        const indexes: IndexInfo[] = result.rows.map((row: any) => ({
          index_name: row.index_name,
          is_primary: row.is_primary || false,
          is_unique: row.is_unique || false,
          definition: row.definition || '',
          size: row.size || ''
        }));
        setTableIndexes(prev => ({ ...prev, [tableName]: indexes }));
        updateCache(undefined, undefined, { [tableName]: indexes });
      }
    } catch (err) {
      console.error('Failed to fetch indexes:', err);
      toast.error(`${tableName}のインデックス情報の取得に失敗しました`);
    }
  };

  // SELECT文を生成
  const generateSelectQuery = async (tableName: string) => {
    try {
      const query = await invoke<string>('generate_select_query', { tableName });
      openQueryTab(query);
      toast.success('SELECT文を生成しました');
    } catch (err) {
      console.error('Failed to generate query:', err);
      toast.error('クエリ生成に失敗しました');
    }
  };

  // クイックアクション
  const handleQuickAction = (action: string, tableName: string) => {
    switch (action) {
      case 'select':
        generateSelectQuery(tableName);
        break;
      case 'view':
        openTableTab(tableName);
        break;
      case 'copy':
        navigator.clipboard.writeText(tableName);
        toast.success('テーブル名をコピーしました');
        break;
      case 'count':
        openQueryTab(`SELECT COUNT(*) AS total_count FROM ${tableName};`);
        break;
      case 'sample':
        openQueryTab(`SELECT * FROM ${tableName} LIMIT 10;`);
        break;
      case 'indexes':
        setShowIndexes(prev => {
          const newSet = new Set(prev);
          if (newSet.has(tableName)) {
            newSet.delete(tableName);
          } else {
            newSet.add(tableName);
            if (!tableIndexes[tableName]) {
              fetchTableIndexes(tableName);
            }
          }
          return newSet;
        });
        break;
    }
  };

  // テーブルの展開/折りたたみ
  const toggleTable = async (tableName: string) => {
    const newExpanded = new Set(expandedTables);
    if (newExpanded.has(tableName)) {
      newExpanded.delete(tableName);
      setShowIndexes(prev => {
        const newSet = new Set(prev);
        newSet.delete(tableName);
        return newSet;
      });
    } else {
      newExpanded.add(tableName);
      // カラム情報がまだない場合は取得
      if (!tableColumns[tableName]) {
        await fetchTableColumns(tableName);
      }
      // インデックス情報も取得
      if (!tableIndexes[tableName]) {
        await fetchTableIndexes(tableName);
      }
    }
    setExpandedTables(newExpanded);
  };

  // 接続時にテーブル一覧を取得
  useEffect(() => {
    if (currentProfile) {
      fetchTables(true);
    } else {
      setTables([]);
      setTableColumns({});
      setTableIndexes({});
      setMetadataCache(null);
    }
  }, [currentProfile]);

  if (!currentProfile) {
    return (
      <div className="p-4 text-center text-muted-foreground">
        <Database className="w-8 h-8 mx-auto mb-2 opacity-50" />
        <p className="text-sm">データベースに接続してください</p>
      </div>
    );
  }

  return (
    <TooltipProvider>
      <div className="h-full flex flex-col">
        <div className="p-2 border-b flex items-center justify-between">
          <h3 className="text-sm font-semibold">データベース</h3>
          <div className="flex items-center gap-1">
            {metadataCache && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="text-xs text-muted-foreground px-2">
                    <Zap className="w-3 h-3 inline" />
                  </div>
                </TooltipTrigger>
                <TooltipContent>
                  <p>キャッシュ使用中</p>
                </TooltipContent>
              </Tooltip>
            )}
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={() => fetchTables(false)}
              disabled={loading}
            >
              {loading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <RefreshCw className="w-4 h-4" />
              )}
            </Button>
          </div>
        </div>

        <ScrollArea className="flex-1">
          <div className="p-2">
            {loading && tables.length === 0 ? (
              <div className="flex items-center gap-2 text-sm text-muted-foreground p-2">
                <Loader2 className="w-4 h-4 animate-spin" />
                読み込み中...
              </div>
            ) : error ? (
              <div className="text-sm text-destructive p-2">
                <div className="flex items-center gap-2">
                  <AlertCircle className="w-4 h-4" />
                  <span>エラー</span>
                </div>
                <p className="mt-1 text-xs">{error}</p>
              </div>
            ) : tables.length === 0 ? (
              <div className="text-sm text-muted-foreground p-2">
                テーブルがありません
              </div>
            ) : (
              <div className="space-y-0.5">
                {tables.map((table) => {
                  const isExpanded = expandedTables.has(table.name);
                  const columns = tableColumns[table.name] || [];
                  const indexes = tableIndexes[table.name] || [];
                  const showIndex = showIndexes.has(table.name);

                  return (
                    <div key={table.name}>
                      <div className="flex items-center gap-1 p-1 hover:bg-accent rounded text-sm group">
                        <button
                          className="p-0.5"
                          onClick={() => toggleTable(table.name)}
                        >
                          <ChevronRight
                            className={cn(
                              "w-3 h-3 transition-transform",
                              isExpanded && "rotate-90"
                            )}
                          />
                        </button>
                        <div
                          className="flex items-center gap-1 flex-1 cursor-pointer"
                          onClick={() => openTableTab(table.name)}
                        >
                          <Table className="w-4 h-4 text-muted-foreground" />
                          <span className="flex-1">{table.name}</span>
                          {table.type === 'view' && (
                            <span className="text-xs text-muted-foreground">VIEW</span>
                          )}
                        </div>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-6 w-6 opacity-0 group-hover:opacity-100"
                            >
                              <MoreVertical className="w-4 h-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => handleQuickAction('view', table.name)}>
                              <FileText className="w-4 h-4 mr-2" />
                              データを表示
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => handleQuickAction('select', table.name)}>
                              <Play className="w-4 h-4 mr-2" />
                              SELECT文を生成
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => handleQuickAction('sample', table.name)}>
                              <Zap className="w-4 h-4 mr-2" />
                              サンプル (10件)
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => handleQuickAction('count', table.name)}>
                              <Hash className="w-4 h-4 mr-2" />
                              件数を取得
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem onClick={() => handleQuickAction('indexes', table.name)}>
                              <Info className="w-4 h-4 mr-2" />
                              インデックス情報
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => handleQuickAction('copy', table.name)}>
                              <Copy className="w-4 h-4 mr-2" />
                              テーブル名をコピー
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>

                      {isExpanded && (
                        <div className="ml-5 space-y-0.5">
                          {columns.length === 0 ? (
                            <div className="text-xs text-muted-foreground p-1">
                              <Loader2 className="w-3 h-3 animate-spin inline mr-1" />
                              読み込み中...
                            </div>
                          ) : (
                            <>
                              {columns.map((column) => (
                                <div
                                  key={column.name}
                                  className="flex items-center gap-1 p-0.5 hover:bg-accent/50 rounded text-xs"
                                >
                                  {column.primary_key ? (
                                    <Key className="w-3 h-3 text-yellow-500" />
                                  ) : (
                                    <Columns className="w-3 h-3 text-muted-foreground" />
                                  )}
                                  <span className="font-mono">{column.name}</span>
                                  <span className="text-muted-foreground">
                                    {column.type}
                                  </span>
                                  {!column.nullable && (
                                    <span className="text-xs text-orange-500">NOT NULL</span>
                                  )}
                                </div>
                              ))}
                              
                              {showIndex && indexes.length > 0 && (
                                <div className="mt-2 pt-2 border-t">
                                  <div className="text-xs font-semibold text-muted-foreground mb-1">
                                    インデックス
                                  </div>
                                  {indexes.map((index) => (
                                    <div
                                      key={index.index_name}
                                      className="flex items-center gap-1 p-0.5 hover:bg-accent/50 rounded text-xs"
                                    >
                                      <Key className={cn(
                                        "w-3 h-3",
                                        index.is_primary ? "text-yellow-500" :
                                        index.is_unique ? "text-blue-500" : "text-muted-foreground"
                                      )} />
                                      <span className="font-mono flex-1">{index.index_name}</span>
                                      {index.is_primary && (
                                        <span className="text-xs text-yellow-500">PK</span>
                                      )}
                                      {index.is_unique && !index.is_primary && (
                                        <span className="text-xs text-blue-500">UNIQUE</span>
                                      )}
                                      {index.size && (
                                        <span className="text-xs text-muted-foreground">{index.size}</span>
                                      )}
                                    </div>
                                  ))}
                                </div>
                              )}
                            </>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </ScrollArea>
      </div>
    </TooltipProvider>
  );
}