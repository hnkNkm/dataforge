export type DatabaseType = "postgresql" | "mysql" | "sqlite";

export interface ConnectionProfile {
  id: string;
  name: string;
  database_type: DatabaseType;
  host?: string;
  port?: number;
  database: string;
  username?: string;
  ssl_mode?: string;
  color?: string;
  icon?: string;
  created_at: string;
  updated_at: string;
  last_connected?: string;
}

export interface CreateProfileRequest {
  name: string;
  database_type: DatabaseType;
  host?: string;
  port?: number;
  database: string;
  username?: string;
  password?: string;
  ssl_mode?: string;
  color?: string;
  icon?: string;
}

export interface UpdateProfileRequest extends CreateProfileRequest {
  id: string;
}

export interface ConnectionState {
  isConnected: boolean;
  currentProfile?: ConnectionProfile;
  connectionMessage?: string;
}