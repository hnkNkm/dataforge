import React, { useState, useRef } from 'react';
import Editor, { Monaco } from '@monaco-editor/react';
import { editor } from 'monaco-editor';
import { invoke } from '@tauri-apps/api/core';
import { Play, Loader2 } from 'lucide-react';
import { Button } from './ui/button';
import { useConnectionStore } from '@/stores/connectionStore';
import { toast } from 'sonner';
import { useTheme } from 'next-themes';

interface MonacoQueryEditorProps {
  initialContent?: string;
  onContentChange?: (content: string) => void;
}

export function MonacoQueryEditor({ initialContent = 'SELECT * FROM ', onContentChange }: MonacoQueryEditorProps) {
  const [query, setQuery] = useState(initialContent);
  const [results, setResults] = useState<any>(null);
  const [currentResultIndex, setCurrentResultIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [executionTime, setExecutionTime] = useState<number | null>(null);
  const { currentProfile } = useConnectionStore();
  const { resolvedTheme } = useTheme();
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  const handleEditorDidMount = (editor: editor.IStandaloneCodeEditor, monaco: Monaco) => {
    editorRef.current = editor;

    // Register SQL completion provider
    monaco.languages.registerCompletionItemProvider('sql', {
      provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn,
        };

        // Basic SQL keywords
        const keywords = [
          'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'DROP',
          'ALTER', 'TABLE', 'INDEX', 'VIEW', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER',
          'ON', 'AS', 'ORDER', 'BY', 'GROUP', 'HAVING', 'DISTINCT', 'LIMIT', 'OFFSET',
          'AND', 'OR', 'NOT', 'IN', 'EXISTS', 'BETWEEN', 'LIKE', 'IS', 'NULL',
          'VALUES', 'INTO', 'SET', 'PRIMARY', 'KEY', 'FOREIGN', 'REFERENCES',
          'CASCADE', 'RESTRICT', 'DEFAULT', 'UNIQUE', 'CHECK', 'CONSTRAINT'
        ];

        const suggestions = keywords.map(keyword => ({
          label: keyword,
          kind: monaco.languages.CompletionItemKind.Keyword,
          insertText: keyword,
          range: range,
        }));

        return { suggestions };
      },
    });

    // Add keyboard shortcut for execution
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      handleExecute();
    });

    // Configure editor options
    editor.updateOptions({
      minimap: { enabled: false },
      fontSize: 14,
      lineNumbers: 'on',
      roundedSelection: false,
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      automaticLayout: true,
      suggestOnTriggerCharacters: true,
      quickSuggestions: {
        other: true,
        comments: false,
        strings: false,
      },
    });
  };

  const handleExecute = async () => {
    if (!currentProfile) {
      toast.error('データベースに接続してください');
      return;
    }

    const currentQuery = editorRef.current?.getValue() || query;
    if (!currentQuery.trim()) {
      toast.error('クエリを入力してください');
      return;
    }

    setLoading(true);
    setError(null);
    setResults(null);
    const startTime = Date.now();

    try {
      const result = await invoke<any>('execute_query', { query: currentQuery });
      const endTime = Date.now();
      setExecutionTime(endTime - startTime);

      if (result) {
        setResults(result);
        setCurrentResultIndex(0);

        // Check if multiple results
        if (result.results) {
          const totalRows = result.results.reduce((sum: number, r: any) =>
            sum + (r.rows?.length || r.rows_affected || 0), 0);
          toast.success(`${result.results.length}個のステートメントが実行されました (${totalRows}行)`);
        } else {
          toast.success(`クエリが正常に実行されました (${result.rows?.length || 0}行)`);
        }
      }
    } catch (err) {
      console.error('Query execution failed:', err);
      setError(err instanceof Error ? err.message : 'クエリの実行に失敗しました');
      toast.error('クエリの実行に失敗しました');
    } finally {
      setLoading(false);
    }
  };

  const handleEditorChange = (value: string | undefined) => {
    if (value !== undefined) {
      setQuery(value);
      if (onContentChange) {
        onContentChange(value);
      }
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

        <div className="flex-1 min-h-0">
          <Editor
            height="100%"
            defaultLanguage="sql"
            defaultValue={initialContent}
            theme={resolvedTheme === 'dark' ? 'vs-dark' : 'light'}
            onChange={handleEditorChange}
            onMount={handleEditorDidMount}
            options={{
              minimap: { enabled: false },
              fontSize: 14,
              lineNumbers: 'on',
              roundedSelection: false,
              scrollBeyondLastLine: false,
              wordWrap: 'on',
              automaticLayout: true,
            }}
          />
        </div>
      </div>

      {/* 結果エリア */}
      <div className="flex-1 flex flex-col min-h-0 border-t">
        <div className="border-b px-3 py-2 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <h3 className="text-sm font-semibold">結果</h3>
            {results?.results && results.results.length > 1 && (
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setCurrentResultIndex(Math.max(0, currentResultIndex - 1))}
                  disabled={currentResultIndex === 0}
                  className="text-xs px-2 py-1 rounded hover:bg-accent disabled:opacity-50"
                >
                  ←前
                </button>
                <span className="text-xs text-muted-foreground">
                  {currentResultIndex + 1} / {results.results.length}
                </span>
                <button
                  onClick={() => setCurrentResultIndex(Math.min(results.results.length - 1, currentResultIndex + 1))}
                  disabled={currentResultIndex === results.results.length - 1}
                  className="text-xs px-2 py-1 rounded hover:bg-accent disabled:opacity-50"
                >
                  次→
                </button>
              </div>
            )}
          </div>
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
            (() => {
              // Get current result to display
              const currentResult = results.results ? results.results[currentResultIndex] : results;

              if (!currentResult) return null;

              // Display command results
              if (currentResult.type === 'command') {
                return (
                  <div className="p-4">
                    <div className="text-sm font-mono mb-2">{currentResult.statement}</div>
                    <div className="text-sm text-muted-foreground">
                      {currentResult.rows_affected} 行が影響を受けました
                    </div>
                  </div>
                );
              }

              // Display query results
              const displayData = currentResult.rows || results.rows;
              const displayColumns = currentResult.columns || results.columns;

              if (!displayData || !displayColumns) {
                return (
                  <div className="p-4 text-center text-muted-foreground">
                    結果がありません
                  </div>
                );
              }

              return (
                <div className="w-full">
                  <table className="w-full border-collapse">
                    <thead className="sticky top-0 bg-background">
                      <tr className="border-b">
                        {displayColumns.map((col: any, index: number) => (
                          <th key={index} className="text-left p-2 text-sm font-medium border-r last:border-r-0">
                            {col.name}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {displayData.map((row: any, rowIndex: number) => (
                        <tr key={rowIndex} className="border-b hover:bg-accent/50">
                          {displayColumns.map((col: any, colIndex: number) => (
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
                  {displayData.length === 0 && (
                    <div className="p-4 text-center text-muted-foreground">
                      結果がありません
                    </div>
                  )}
                </div>
              );
            })()
          ) : (
            <div className="p-4 text-center text-muted-foreground">
              クエリを実行すると結果がここに表示されます
            </div>
          )}
        </div>

        {results && (
          <div className="border-t px-3 py-2">
            <span className="text-xs text-muted-foreground">
              {(() => {
                if (results.results) {
                  const current = results.results[currentResultIndex];
                  if (current?.type === 'command') {
                    return `${current.rows_affected} 行が影響を受けました`;
                  } else if (current?.rows) {
                    return `${current.rows.length} 行が返されました`;
                  }
                }
                return `${results.rows?.length || 0} 行が返されました`;
              })()}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}