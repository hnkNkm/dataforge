import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ConnectionProfile, ConnectionState } from '../types/profile';

interface ConnectionStoreState extends ConnectionState {
  // Actions
  connectWithProfile: (profileId: string) => Promise<void>;
  disconnect: () => Promise<void>;
  testConnection: () => Promise<boolean>;
  setConnectionState: (state: Partial<ConnectionState>) => void;
}

export const useConnectionStore = create<ConnectionStoreState>((set) => ({
  isConnected: false,
  currentProfile: undefined,
  connectionMessage: undefined,

  connectWithProfile: async (profileId: string) => {
    try {
      const message = await invoke<string>('connect_with_profile', {
        profileId
      });

      // Get the profile details
      const profile = await invoke<ConnectionProfile>('get_profile', {
        id: profileId
      });

      set({
        isConnected: true,
        currentProfile: profile,
        connectionMessage: message
      });
    } catch (error) {
      set({
        isConnected: false,
        currentProfile: undefined,
        connectionMessage: error instanceof Error ? error.message : String(error)
      });
      throw error;
    }
  },

  disconnect: async () => {
    try {
      const message = await invoke<string>('disconnect_database');
      set({
        isConnected: false,
        currentProfile: undefined,
        connectionMessage: message
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({
        connectionMessage: errorMessage
      });
      throw error;
    }
  },

  testConnection: async () => {
    try {
      const result = await invoke<boolean>('test_database_connection_adapter');
      return result;
    } catch (error) {
      return false;
    }
  },

  setConnectionState: (state: Partial<ConnectionState>) => {
    set(state);
  }
}));