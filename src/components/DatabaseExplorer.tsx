import React, { useEffect, useState } from 'react';
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
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
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

export function DatabaseExplorer() {
  const { currentProfile } = useConnectionStore();
  const { openTableTab } = useTabStore();
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());
  const [tableColumns, setTableColumns] = useState<Record<string, ColumnInfo[]>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // テーブル一覧を取得
  const fetchTables = async () => {
    if (!currentProfile) return;

    setLoading(true);
    setError(null);
    try {
      const result = await invoke<any>('list_database_tables');
      console.log('Raw tables result:', result);
      console.log('Result type:', typeof result);
      console.log('Is Array:', Array.isArray(result));

      // 結果の形式に応じて処理
      if (Array.isArray(result)) {
        console.log('Processing as array, length:', result.length);
        // PostgreSQLからの結果はTableInfo型の配列
        const processedTables = result.map((table: any) => {
          console.log('Processing table:', table);
          return {
            name: table.name,
            type: table.table_type === 'TABLE' ? 'table' : 'view',
            schema: table.schema
          };
        });
        console.log('Processed tables:', processedTables);
        setTables(processedTables);
      } else if (result && result.tables) {
        console.log('Processing as object with tables property');
        setTables(result.tables);
      } else {
        console.log('Unexpected result format');
        setTables([]);
      }

      if (result && Array.isArray(result) && result.length > 0) {
        toast.success(`${result.length}個のテーブルを取得しました`);
      } else {
        toast.info('テーブルが見つかりませんでした');
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
            DATA_TYPE as data_type,
            IS_NULLABLE as is_nullable,
            COLUMN_DEFAULT as column_default
           FROM information_schema.columns
           WHERE TABLE_NAME = '${tableName}'
           ORDER BY ORDINAL_POSITION`
        : `PRAGMA table_info(${tableName})`;

      const result = await invoke<any>('execute_query', { query });
      console.log('Columns result:', result);

      if (result.rows) {
        const columns = result.rows.map((row: any) => ({
          name: row.column_name || row.name,
          type: row.data_type || row.type,
          nullable: row.is_nullable === 'YES',
          default: row.column_default || row.default
        }));
        setTableColumns(prev => ({
          ...prev,
          [tableName]: columns
        }));
      }
    } catch (err) {
      console.error('Failed to fetch columns:', err);
      toast.error(`${tableName}のカラム情報の取得に失敗しました`);
    }
  };

  // テーブルの展開/折りたたみ
  const toggleTable = async (tableName: string) => {
    const newExpanded = new Set(expandedTables);
    if (newExpanded.has(tableName)) {
      newExpanded.delete(tableName);
    } else {
      newExpanded.add(tableName);
      // カラム情報がまだない場合は取得
      if (!tableColumns[tableName]) {
        await fetchTableColumns(tableName);
      }
    }
    setExpandedTables(newExpanded);
  };

  // 接続時にテーブル一覧を取得
  useEffect(() => {
    if (currentProfile) {
      fetchTables();
    } else {
      setTables([]);
      setTableColumns({});
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
    <div className="h-full flex flex-col">
      <div className="p-2 border-b flex items-center justify-between">
        <h3 className="text-sm font-semibold">データベース</h3>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={fetchTables}
          disabled={loading}
        >
          {loading ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <RefreshCw className="w-4 h-4" />
          )}
        </Button>
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

                return (
                  <div key={table.name}>
                    <div
                      className="flex items-center gap-1 p-1 hover:bg-accent rounded text-sm"
                    >
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
                    </div>

                    {isExpanded && (
                      <div className="ml-5 space-y-0.5">
                        {columns.length === 0 ? (
                          <div className="text-xs text-muted-foreground p-1">
                            <Loader2 className="w-3 h-3 animate-spin inline mr-1" />
                            読み込み中...
                          </div>
                        ) : (
                          columns.map((column) => (
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
                          ))
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
  );
}