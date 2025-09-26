import { useEffect, useState } from 'react';
import { useProfileStore } from '../stores/profileStore';
import { useConnectionStore } from '../stores/connectionStore';
import { ConnectionProfile } from '../types/profile';
import {
  Database,
  Plus,
  Trash2,
  Edit2,
  Play,
  Clock,
  Server,
  HardDrive,
  Loader2,
  AlertTriangle
} from 'lucide-react';

const getDatabaseIcon = (type: string) => {
  switch (type) {
    case 'postgresql':
      return <Database className="w-5 h-5 text-blue-500" />;
    case 'mysql':
      return <Server className="w-5 h-5 text-orange-500" />;
    case 'sqlite':
      return <HardDrive className="w-5 h-5 text-green-500" />;
    default:
      return <Database className="w-5 h-5 text-gray-500" />;
  }
};

// 削除確認ダイアログコンポーネント
interface DeleteConfirmDialogProps {
  isOpen: boolean;
  profileName: string;
  onConfirm: () => void;
  onCancel: () => void;
}

function DeleteConfirmDialog({ isOpen, profileName, onConfirm, onCancel }: DeleteConfirmDialogProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
        <div className="flex items-start gap-3 mb-4">
          <AlertTriangle className="w-6 h-6 text-yellow-500 flex-shrink-0 mt-0.5" />
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
              プロファイルの削除
            </h3>
            <p className="text-gray-600 dark:text-gray-400">
              プロファイル「{profileName}」を削除しますか？この操作は取り消せません。
            </p>
          </div>
        </div>
        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-md transition-colors"
          >
            キャンセル
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md transition-colors"
          >
            削除
          </button>
        </div>
      </div>
    </div>
  );
}

interface ProfileCardProps {
  profile: ConnectionProfile;
  onConnect: (profileId: string) => void;
  onCancelConnection: () => void;
  onEdit: (profile: ConnectionProfile) => void;
  onDelete: (profileId: string) => Promise<void>;
  isConnected: boolean;
  isConnecting: boolean;
  connectingProfileId: string | null;
}

function ProfileCard({
  profile,
  onConnect,
  onCancelConnection,
  onEdit,
  onDelete,
  isConnected,
  isConnecting,
  connectingProfileId
}: ProfileCardProps) {
  const [isDeleting, setIsDeleting] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  const handleDelete = () => {
    setShowDeleteDialog(true);
  };

  const handleConfirmDelete = async () => {
    setShowDeleteDialog(false);
    setIsDeleting(true);
    try {
      await onDelete(profile.id);
    } catch (error) {
      console.error('Failed to delete profile:', error);
      alert(`プロファイルの削除に失敗しました: ${error}`);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleCancelDelete = () => {
    setShowDeleteDialog(false);
  };

  return (
    <div className={`
      relative p-4 bg-white dark:bg-gray-800 rounded-lg border-2
      ${isConnected
        ? 'border-blue-500 shadow-lg'
        : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
      }
      transition-all duration-200
    `}>
      {isConnected && (
        <div className="absolute -top-2 -right-2 px-2 py-1 bg-blue-500 text-white text-xs font-medium rounded-full">
          接続中
        </div>
      )}

      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-3">
          {getDatabaseIcon(profile.database_type)}
          <div>
            <h3 className="font-semibold text-gray-900 dark:text-gray-100">
              {profile.name}
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {profile.database_type.toUpperCase()}
            </p>
          </div>
        </div>
      </div>

      <div className="space-y-1 mb-3">
        <div className="text-sm text-gray-600 dark:text-gray-400">
          <span className="font-medium">データベース:</span> {profile.database}
        </div>
        {profile.host && (
          <div className="text-sm text-gray-600 dark:text-gray-400">
            <span className="font-medium">ホスト:</span> {profile.host}:{profile.port || 'default'}
          </div>
        )}
        {profile.username && (
          <div className="text-sm text-gray-600 dark:text-gray-400">
            <span className="font-medium">ユーザー:</span> {profile.username}
          </div>
        )}
      </div>

      {profile.last_connected && (
        <div className="flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400 mb-3">
          <Clock className="w-3 h-3" />
          最終接続: {new Date(profile.last_connected).toLocaleString('ja-JP')}
        </div>
      )}

      <div className="flex items-center gap-2">
        {isConnecting && connectingProfileId === profile.id ? (
          <button
            onClick={onCancelConnection}
            className="flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-md font-medium text-sm bg-red-600 hover:bg-red-700 text-white transition-colors"
          >
            <Loader2 className="w-4 h-4 animate-spin" />
            接続中... (キャンセル)
          </button>
        ) : (
          <button
            onClick={() => onConnect(profile.id)}
            disabled={isConnected || (isConnecting && connectingProfileId !== profile.id)}
            className={`
              flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-md font-medium text-sm
              ${isConnected || (isConnecting && connectingProfileId !== profile.id)
                ? 'bg-gray-100 dark:bg-gray-700 text-gray-400 cursor-not-allowed'
                : 'bg-blue-600 hover:bg-blue-700 text-white'
              }
              transition-colors
            `}
          >
            <Play className="w-4 h-4" />
            接続
          </button>
        )}

        <button
          onClick={() => onEdit(profile)}
          className="p-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors"
        >
          <Edit2 className="w-4 h-4" />
        </button>

        <button
          onClick={handleDelete}
          disabled={isDeleting || isConnected}
          className="p-2 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          title={isConnected ? "接続中のプロファイルは削除できません" : "プロファイルを削除"}
        >
          {isDeleting ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Trash2 className="w-4 h-4" />
          )}
        </button>
      </div>

      {/* 削除確認ダイアログ */}
      <DeleteConfirmDialog
        isOpen={showDeleteDialog}
        profileName={profile.name}
        onConfirm={handleConfirmDelete}
        onCancel={handleCancelDelete}
      />
    </div>
  );
}

interface ConnectionProfilesProps {
  onCreateNew: () => void;
  onEditProfile: (profile: ConnectionProfile) => void;
}

export function ConnectionProfiles({ onCreateNew, onEditProfile }: ConnectionProfilesProps) {
  const { profiles, isLoading, loadProfiles, deleteProfile } = useProfileStore();
  const { connectWithProfile, currentProfile, isConnecting, cancelConnection } = useConnectionStore();
  const [connectingProfileId, setConnectingProfileId] = useState<string | null>(null);

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  const handleConnect = async (profileId: string) => {
    setConnectingProfileId(profileId);
    try {
      await connectWithProfile(profileId);
    } catch (error) {
      console.error('Failed to connect:', error);
      alert(`接続に失敗しました: ${error}`);
    } finally {
      setConnectingProfileId(null);
    }
  };

  const handleCancelConnection = async () => {
    try {
      await cancelConnection();
      setConnectingProfileId(null);
    } catch (error) {
      console.error('Failed to cancel connection:', error);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="w-8 h-8 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
          接続プロファイル
        </h2>
        <button
          onClick={onCreateNew}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-medium transition-colors"
        >
          <Plus className="w-4 h-4" />
          新規作成
        </button>
      </div>

      {profiles.length === 0 ? (
        <div className="text-center py-12">
          <Database className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            接続プロファイルがありません
          </p>
          <button
            onClick={onCreateNew}
            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-medium transition-colors"
          >
            <Plus className="w-4 h-4" />
            最初のプロファイルを作成
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {profiles.map((profile) => (
            <ProfileCard
              key={profile.id}
              profile={profile}
              onConnect={handleConnect}
              onCancelConnection={handleCancelConnection}
              onEdit={onEditProfile}
              onDelete={deleteProfile}
              isConnected={currentProfile?.id === profile.id}
              isConnecting={isConnecting}
              connectingProfileId={connectingProfileId}
            />
          ))}
        </div>
      )}

      {isConnecting && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 flex items-center gap-3">
            <Loader2 className="w-6 h-6 animate-spin text-blue-600" />
            <span className="text-gray-700 dark:text-gray-300">接続中...</span>
          </div>
        </div>
      )}
    </div>
  );
}