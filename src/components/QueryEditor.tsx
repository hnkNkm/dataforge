import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Play, Save, Loader2 } from 'lucide-react';
import { Button } from './ui/button';
import { useConnectionStore } from '@/stores/connectionStore';
import { toast } from 'sonner';

interface QueryEditorProps {
  initialContent?: string;
  onContentChange?: (content: string) => void;
}

export function QueryEditor({ initialContent = 'SELECT * FROM ', onContentChange }: QueryEditorProps) {
  const [query, setQuery] = useState(initialContent);
  const [results, setResults] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [executionTime, setExecutionTime] = useState<number | null>(null);
  const { currentProfile } = useConnectionStore();

  const handleExecute = async () => {
    if (!currentProfile) {
      toast.error('データベースに接続してください');
      return;
    }

    if (!query.trim()) {
      toast.error('クエリを入力してください');
      return;
    }

    setLoading(true);
    setError(null);
    setResults(null);
    const startTime = Date.now();

    try {
      const result = await invoke<any>('execute_query', { query });
      const endTime = Date.now();
      setExecutionTime(endTime - startTime);

      if (result) {
        setResults(result);
        toast.success(`クエリが正常に実行されました (${result.rows?.length || 0}行)`);
      }
    } catch (err) {
      console.error('Query execution failed:', err);
      setError(err instanceof Error ? err.message : 'クエリの実行に失敗しました');
      toast.error('クエリの実行に失敗しました');
    } finally {
      setLoading(false);
    }
  };

  const handleQueryChange = (value: string) => {
    setQuery(value);
    if (onContentChange) {
      onContentChange(value);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    // Cmd/Ctrl + Enter でクエリ実行
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault();
      handleExecute();
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* エディタエリア */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="border-b p-2 flex items-center justify-between">
          <div className="text-sm text-muted-foreground">
            SQLエディター
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              onClick={handleExecute}
              disabled={loading || !currentProfile}
            >
              {loading ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  実行中...
                </>
              ) : (
                <>
                  <Play className="w-4 h-4 mr-2" />
                  実行 (⌘Enter)
                </>
              )}
            </Button>
          </div>
        </div>

        <div className="flex-1 p-2 min-h-0">
          <textarea
            className="w-full h-full bg-muted/20 border rounded-md p-3 resize-none outline-none font-mono text-sm focus:ring-2 focus:ring-primary/20"
            placeholder="SELECT * FROM ..."
            value={query}
            onChange={(e) => handleQueryChange(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={loading}
          />
        </div>
      </div>

      {/* 結果エリア */}
      <div className="flex-1 flex flex-col min-h-0 border-t">
        <div className="border-b px-3 py-2 flex items-center justify-between">
          <h3 className="text-sm font-semibold">結果</h3>
          {executionTime !== null && (
            <span className="text-xs text-muted-foreground">
              実行時間: {executionTime}ms
            </span>
          )}
        </div>

        <div className="flex-1 overflow-auto">
          {error ? (
            <div className="p-4">
              <div className="text-sm text-destructive">{error}</div>
            </div>
          ) : results ? (
            <div className="w-full">
              <table className="w-full border-collapse">
                <thead className="sticky top-0 bg-background">
                  <tr className="border-b">
                    {results.columns?.map((col: any, index: number) => (
                      <th key={index} className="text-left p-2 text-sm font-medium border-r last:border-r-0">
                        {col.name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {results.rows?.map((row: any, rowIndex: number) => (
                    <tr key={rowIndex} className="border-b hover:bg-accent/50">
                      {results.columns?.map((col: any, colIndex: number) => (
                        <td key={colIndex} className="p-2 text-sm border-r last:border-r-0">
                          {row[col.name] !== null ? (
                            String(row[col.name])
                          ) : (
                            <span className="text-muted-foreground italic">NULL</span>
                          )}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
              {results.rows?.length === 0 && (
                <div className="p-4 text-center text-muted-foreground">
                  結果がありません
                </div>
              )}
            </div>
          ) : (
            <div className="p-4 text-center text-muted-foreground">
              クエリを実行すると結果がここに表示されます
            </div>
          )}
        </div>

        {results && (
          <div className="border-t px-3 py-2">
            <span className="text-xs text-muted-foreground">
              {results.rows?.length || 0} 行が返されました
            </span>
          </div>
        )}
      </div>
    </div>
  );
}