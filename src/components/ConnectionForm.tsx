import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Database, CheckCircle, XCircle, Loader2, Eye, EyeOff, Save } from "lucide-react";
import { useProfileStore } from "../stores/profileStore";
import { ConnectionProfile } from "../types/profile";

type DatabaseType = "postgresql" | "mysql" | "sqlite";

interface ConnectionFormData {
  database_type: DatabaseType;
  host: string;
  port: string;
  database: string;
  username: string;
  password: string;
  name: string;
}

interface ConnectionFormProps {
  editingProfile?: ConnectionProfile | null;
  onSaveSuccess?: () => void;
  isEditMode?: boolean;
}

export function ConnectionForm({ editingProfile, onSaveSuccess, isEditMode }: ConnectionFormProps) {
  const { createProfile, updateProfile } = useProfileStore();
  const [formData, setFormData] = useState<ConnectionFormData>({
    name: "",
    database_type: "postgresql",
    host: "localhost",
    port: "5433",
    database: "dataforge_dev",
    username: "dataforge",
    password: "dataforge_dev",
  });

  useEffect(() => {
    if (editingProfile) {
      setFormData({
        name: editingProfile.name,
        database_type: editingProfile.database_type as DatabaseType,
        host: editingProfile.host || "localhost",
        port: editingProfile.port?.toString() || "",
        database: editingProfile.database,
        username: editingProfile.username || "",
        password: "", // Password is not returned for security
      });
    }
  }, [editingProfile]);

  const [isConnecting, setIsConnecting] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<{
    status: "idle" | "success" | "error";
    message: string;
  }>({
    status: "idle",
    message: "",
  });

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target;

    // Update port when database type changes
    if (name === "database_type") {
      const defaultPorts: Record<DatabaseType, string> = {
        postgresql: "5433",
        mysql: "3306",
        sqlite: "",
      };

      setFormData({
        ...formData,
        database_type: value as DatabaseType,
        port: defaultPorts[value as DatabaseType],
        // Set appropriate defaults for SQLite
        ...(value === "sqlite" ? {
          host: "",
          database: "database/sqlite/dataforge.db",
          username: "",
          password: "",
        } : {}),
      });
    } else {
      setFormData({
        ...formData,
        [name]: value,
      });
    }
  };

  const saveProfile = async () => {
    setIsConnecting(true);
    setConnectionStatus({ status: "idle", message: "" });

    try {
      const request = {
        name: formData.name,
        database_type: formData.database_type,
        host: formData.host || null,
        port: formData.port ? parseInt(formData.port) : null,
        database: formData.database,
        username: formData.username || null,
        password: formData.password || null,
      };

      if (isEditMode && editingProfile) {
        await updateProfile({ id: editingProfile.id, ...request });
        setConnectionStatus({
          status: "success",
          message: "プロファイルを更新しました",
        });
      } else {
        await createProfile(request);
        setConnectionStatus({
          status: "success",
          message: "プロファイルを保存しました",
        });
      }

      if (onSaveSuccess) {
        setTimeout(onSaveSuccess, 1000);
      }
    } catch (error) {
      setConnectionStatus({
        status: "error",
        message: String(error),
      });
    } finally {
      setIsConnecting(false);
    }
  };

  const testConnection = async () => {
    setIsConnecting(true);
    setConnectionStatus({ status: "idle", message: "" });

    try {
      // Use the new adapter-based connection
      const result = await invoke<string>("connect_database", {
        request: {
          database_type: formData.database_type,
          host: formData.host || null,
          port: formData.port ? parseInt(formData.port) : null,
          database: formData.database,
          username: formData.username || null,
          password: formData.password || null,
          ssl_mode: null,
        },
      });

      setConnectionStatus({
        status: "success",
        message: result,
      });

      // Also test the connection
      const testResult = await invoke<boolean>("test_database_connection_adapter");
      if (!testResult) {
        setConnectionStatus({
          status: "error",
          message: "Connection established but test failed",
        });
      }
    } catch (error) {
      setConnectionStatus({
        status: "error",
        message: String(error),
      });
    } finally {
      setIsConnecting(false);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
      <div className="flex items-center gap-2 mb-6">
        <Database className="w-6 h-6 text-blue-600 dark:text-blue-400" />
        <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-200">
          {isEditMode ? "接続プロファイルを編集" : "新規接続プロファイル"}
        </h2>
      </div>

      <div className="space-y-4">
        <div>
          <label
            htmlFor="name"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            プロファイル名
          </label>
          <input
            id="name"
            name="name"
            type="text"
            value={formData.name}
            onChange={handleInputChange}
            placeholder="例: 本番環境DB"
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
            required
          />
        </div>

        <div>
          <label
            htmlFor="database_type"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            データベースタイプ
          </label>
          <select
            id="database_type"
            name="database_type"
            value={formData.database_type}
            onChange={handleInputChange}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
          >
            <option value="postgresql">PostgreSQL</option>
            <option value="mysql">MySQL</option>
            <option value="sqlite">SQLite</option>
          </select>
        </div>

        {formData.database_type !== "sqlite" && (
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label
                htmlFor="host"
                className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                ホスト
              </label>
              <input
                id="host"
                name="host"
                type="text"
                value={formData.host}
                onChange={handleInputChange}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
                required={formData.database_type !== "sqlite"}
              />
            </div>

            <div>
              <label
                htmlFor="port"
                className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                ポート
              </label>
              <input
                id="port"
                name="port"
                type="text"
                value={formData.port}
                onChange={handleInputChange}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
                required={formData.database_type !== "sqlite"}
              />
            </div>
          </div>
        )}

        <div>
          <label
            htmlFor="database"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            {formData.database_type === "sqlite" ? "データベースファイルパス" : "データベース名"}
          </label>
          <input
            id="database"
            name="database"
            type="text"
            value={formData.database}
            onChange={handleInputChange}
            placeholder={
              formData.database_type === "sqlite"
                ? "例: database/sqlite/myapp.db"
                : formData.database_type === "mysql"
                ? "例: dataforge_dev"
                : "例: dataforge_dev"
            }
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
          />
        </div>

        {formData.database_type !== "sqlite" && (
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label
                htmlFor="username"
                className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                ユーザー名
              </label>
              <input
                id="username"
                name="username"
                type="text"
                value={formData.username}
                onChange={handleInputChange}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
                required={formData.database_type !== "sqlite"}
              />
            </div>

            <div>
              <label
                htmlFor="password"
                className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                パスワード
              </label>
              <div className="relative">
                <input
                  id="password"
                  name="password"
                  type={showPassword ? "text" : "password"}
                  value={formData.password}
                  onChange={handleInputChange}
                  className="w-full px-3 py-2 pr-10 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                  {showPassword ? (
                    <EyeOff className="w-5 h-5" />
                  ) : (
                    <Eye className="w-5 h-5" />
                  )}
                </button>
              </div>
            </div>
          </div>
        )}

        <div className="flex justify-end gap-3 mt-6">
          <button
            onClick={testConnection}
            disabled={isConnecting}
            className="px-6 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            {isConnecting ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                テスト中...
              </>
            ) : (
              <>
                <Database className="w-4 h-4" />
                接続テスト
              </>
            )}
          </button>
          <button
            onClick={saveProfile}
            disabled={isConnecting || !formData.name || !formData.database}
            className="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            {isConnecting ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                保存中...
              </>
            ) : (
              <>
                <Save className="w-4 h-4" />
                {isEditMode ? "更新" : "保存"}
              </>
            )}
          </button>
        </div>

        {connectionStatus.message && (
          <div
            className={`mt-4 p-4 rounded-md flex items-start gap-3 ${
              connectionStatus.status === "success"
                ? "bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-300"
                : connectionStatus.status === "error"
                ? "bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-300"
                : "bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
            }`}
          >
            {connectionStatus.status === "success" ? (
              <CheckCircle className="w-5 h-5 mt-0.5 flex-shrink-0" />
            ) : connectionStatus.status === "error" ? (
              <XCircle className="w-5 h-5 mt-0.5 flex-shrink-0" />
            ) : null}
            <pre className="whitespace-pre-wrap text-sm">{connectionStatus.message}</pre>
          </div>
        )}
      </div>
    </div>
  );
}