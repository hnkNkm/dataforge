import { useConnectionStore } from '../stores/connectionStore';
import { Database, Circle, AlertCircle, Loader2 } from 'lucide-react';

export function ConnectionStatus() {
  const { isConnected, currentProfile, disconnect } = useConnectionStore();

  const handleDisconnect = async () => {
    try {
      await disconnect();
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  };

  return (
    <div className="flex items-center gap-4 px-4 py-2 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <div className="flex items-center gap-2">
        <Database className="w-5 h-5 text-gray-600 dark:text-gray-400" />
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          接続状態:
        </span>
      </div>

      {isConnected && currentProfile ? (
        <div className="flex items-center gap-3 flex-1">
          <div className="flex items-center gap-2">
            <Circle className="w-2 h-2 fill-green-500 text-green-500" />
            <span className="text-sm text-green-600 dark:text-green-400">
              接続中
            </span>
          </div>

          <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
            <span className="font-medium">{currentProfile.name}</span>
            <span className="text-xs">
              ({currentProfile.database_type} - {currentProfile.database})
            </span>
          </div>

          <button
            onClick={handleDisconnect}
            className="ml-auto px-3 py-1 text-xs font-medium text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md transition-colors"
          >
            切断
          </button>
        </div>
      ) : (
        <div className="flex items-center gap-2">
          <Circle className="w-2 h-2 fill-gray-400 text-gray-400" />
          <span className="text-sm text-gray-500 dark:text-gray-400">
            未接続
          </span>
        </div>
      )}
    </div>
  );
}