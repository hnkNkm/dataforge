import React, { useEffect, useState, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Loader2, RefreshCw, Download, Search, ChevronUp, ChevronDown, ChevronsUpDown } from 'lucide-react';
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
  const [sortColumn, setSortColumn] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
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
    if (!sortedData?.rows || sortedData.rows.length === 0) {
      toast.error('エクスポートするデータがありません');
      return;
    }

    try {
      // CSVデータを生成
      const csvRows: string[] = [];
      
      // BOMを追加（Excelで文字化けを防ぐため）
      const BOM = '\uFEFF';
      
      // ヘッダー行を追加
      const headers = sortedData.columns.map(col => {
        // カンマやダブルクォートを含む場合はダブルクォートで囲む
        const value = col.name;
        if (value.includes(',') || value.includes('"') || value.includes('\n')) {
          return `"${value.replace(/"/g, '""')}"`;
        }
        return value;
      });
      csvRows.push(headers.join(','));
      
      // データ行を追加
      sortedData.rows.forEach(row => {
        const values = sortedData.columns.map(col => {
          const value = row[col.name];
          if (value === null || value === undefined) {
            return '';
          }
          const strValue = String(value);
          // カンマやダブルクォートを含む場合はダブルクォートで囲む
          if (strValue.includes(',') || strValue.includes('"') || strValue.includes('\n')) {
            return `"${strValue.replace(/"/g, '""')}"`;
          }
          return strValue;
        });
        csvRows.push(values.join(','));
      });
      
      const csvContent = BOM + csvRows.join('\n');
      const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
      const filename = `${tableName}_${timestamp}.csv`;
      
      // Blobを作成してダウンロード
      const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      
      toast.success('CSVファイルをエクスポートしました');
    } catch (error) {
      console.error('CSV export failed:', error);
      toast.error('CSVエクスポートに失敗しました');
    }
  };

  // ソート機能
  const handleSort = (columnName: string) => {
    if (sortColumn === columnName) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(columnName);
      setSortDirection('asc');
    }
  };

  // ソートアイコンを取得
  const getSortIcon = (columnName: string) => {
    if (sortColumn !== columnName) {
      return <ChevronsUpDown className="w-3 h-3 opacity-50" />;
    }
    return sortDirection === 'asc' 
      ? <ChevronUp className="w-3 h-3" />
      : <ChevronDown className="w-3 h-3" />;
  };

  // ソート済みデータを計算
  const sortedData = useMemo(() => {
    if (!data?.rows || !sortColumn) return data;

    const sortedRows = [...data.rows].sort((a, b) => {
      const aValue = a[sortColumn] ?? '';
      const bValue = b[sortColumn] ?? '';
      
      // NULL値の処理
      if (aValue === '' && bValue === '') return 0;
      if (aValue === '') return sortDirection === 'asc' ? 1 : -1;
      if (bValue === '') return sortDirection === 'asc' ? -1 : 1;
      
      // 数値判定と比較
      const aNum = parseFloat(aValue);
      const bNum = parseFloat(bValue);
      const isNumeric = !isNaN(aNum) && !isNaN(bNum) && 
                       String(aValue).trim() === String(aNum) && 
                       String(bValue).trim() === String(bNum);
      
      if (isNumeric) {
        return sortDirection === 'asc' ? aNum - bNum : bNum - aNum;
      }
      
      // 日付判定と比較
      const aDate = new Date(aValue);
      const bDate = new Date(bValue);
      const isDate = !isNaN(aDate.getTime()) && !isNaN(bDate.getTime()) &&
                    String(aValue).includes('-') || String(aValue).includes('/');
      
      if (isDate) {
        return sortDirection === 'asc' 
          ? aDate.getTime() - bDate.getTime() 
          : bDate.getTime() - aDate.getTime();
      }
      
      // 文字列比較
      const aStr = String(aValue).toLowerCase();
      const bStr = String(bValue).toLowerCase();
      
      const comparison = aStr.localeCompare(bStr, 'ja');
      return sortDirection === 'asc' ? comparison : -comparison;
    });

    return {
      ...data,
      rows: sortedRows
    };
  }, [data, sortColumn, sortDirection]);

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
              {sortedData?.columns.map((column, index) => (
                <th
                  key={index}
                  className="text-left p-2 font-medium text-sm border-r last:border-r-0 cursor-pointer hover:bg-accent/50 transition-colors"
                  onClick={() => handleSort(column.name)}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div>{column.name}</div>
                      <div className="text-xs text-muted-foreground font-normal">
                        {column.type}
                      </div>
                    </div>
                    <div className="ml-2">
                      {getSortIcon(column.name)}
                    </div>
                  </div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {sortedData?.rows.map((row, rowIndex) => (
              <tr key={rowIndex} className="border-b hover:bg-accent/50">
                {sortedData.columns.map((column, colIndex) => (
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
        {sortedData?.rows.length === 0 && (
          <div className="p-4 text-center text-muted-foreground">
            データがありません
          </div>
        )}
      </div>

      {/* フッター */}
      <div className="border-t p-2 flex items-center justify-between text-sm text-muted-foreground">
        <div>
          {sortedData?.rowCount || 0} 行を表示
          {limit < (sortedData?.rowCount || 0) && ` (最初の${limit}行)`}
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
            disabled={!sortedData || sortedData.rows.length < limit}
          >
            次へ
          </Button>
        </div>
      </div>
    </div>
  );
}