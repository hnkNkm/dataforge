import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Loader2, RefreshCw, Download, Search } from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { useConnectionStore } from '@/stores/connectionStore';
import { toast } from 'sonner';

interface TableViewProps {
  tableName: string;
}

interface TableData {
  columns: Array<{
    name: string;
    type: string;
  }>;
  rows: any[];
  rowCount: number;
}

export function TableView({ tableName }: TableViewProps) {
  const [data, setData] = useState<TableData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [limit, setLimit] = useState(100);
  const [offset, setOffset] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const { currentProfile } = useConnectionStore();

  const loadTableData = async () => {
    if (!currentProfile) {
      setError('データベースに接続されていません');
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // テーブルデータを取得するクエリ
      const query = `SELECT * FROM ${tableName} LIMIT ${limit} OFFSET ${offset}`;
      const result = await invoke<any>('execute_query', { query });

      console.log('Query result:', result);
      console.log('Columns:', result?.columns);
      console.log('First row:', result?.rows?.[0]);

      if (result) {
        setData({
          columns: result.columns || [],
          rows: result.rows || [],
          rowCount: result.rows?.length || 0
        });
      }
    } catch (err) {
      console.error('Failed to load table data:', err);
      setError(err instanceof Error ? err.message : 'データの取得に失敗しました');
      toast.error('テーブルデータの取得に失敗しました');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTableData();
  }, [tableName, limit, offset]);

  const handleRefresh = () => {
    loadTableData();
  };

  const handleExport = () => {
    // CSVエクスポート機能（将来実装）
    toast.info('エクスポート機能は準備中です');
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 animate-spin text-primary" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <p className="text-destructive mb-4">{error}</p>
        <Button onClick={handleRefresh} variant="outline">
          <RefreshCw className="w-4 h-4 mr-2" />
          再試行
        </Button>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* ツールバー */}
      <div className="border-b p-2 flex items-center gap-2">
        <div className="flex-1 flex items-center gap-2">
          <Input
            placeholder="検索..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="max-w-xs"
          />
          <Button size="sm" variant="ghost">
            <Search className="w-4 h-4" />
          </Button>
        </div>
        <Button size="sm" variant="ghost" onClick={handleRefresh}>
          <RefreshCw className="w-4 h-4" />
        </Button>
        <Button size="sm" variant="ghost" onClick={handleExport}>
          <Download className="w-4 h-4" />
        </Button>
      </div>

      {/* テーブル */}
      <div className="flex-1 overflow-auto">
        <table className="w-full border-collapse">
          <thead className="sticky top-0 bg-background border-b">
            <tr>
              {data?.columns.map((column, index) => (
                <th
                  key={index}
                  className="text-left p-2 font-medium text-sm border-r last:border-r-0"
                >
                  <div>{column.name}</div>
                  <div className="text-xs text-muted-foreground font-normal">
                    {column.type}
                  </div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {data?.rows.map((row, rowIndex) => (
              <tr key={rowIndex} className="border-b hover:bg-accent/50">
                {data.columns.map((column, colIndex) => (
                  <td
                    key={colIndex}
                    className="p-2 text-sm border-r last:border-r-0"
                  >
                    <div className="max-w-xs truncate">
                      {(() => {
                        // カラム名のバリエーションを試す
                        const value = row[column.name] ?? row[colIndex];
                        if (value !== null && value !== undefined) {
                          return String(value);
                        }
                        return <span className="text-muted-foreground italic">NULL</span>;
                      })()}
                    </div>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
        {data?.rows.length === 0 && (
          <div className="p-4 text-center text-muted-foreground">
            データがありません
          </div>
        )}
      </div>

      {/* フッター */}
      <div className="border-t p-2 flex items-center justify-between text-sm text-muted-foreground">
        <div>
          {data?.rowCount || 0} 行を表示
          {limit < (data?.rowCount || 0) && ` (最初の${limit}行)`}
        </div>
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setOffset(Math.max(0, offset - limit))}
            disabled={offset === 0}
          >
            前へ
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setOffset(offset + limit)}
            disabled={!data || data.rows.length < limit}
          >
            次へ
          </Button>
        </div>
      </div>
    </div>
  );
}