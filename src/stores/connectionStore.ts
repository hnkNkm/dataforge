import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ConnectionProfile, ConnectionState } from '../types/profile';

interface ConnectionStoreState extends ConnectionState {
  isConnecting: boolean;
  // Actions
  connectWithProfile: (profileId: string) => Promise<void>;
  disconnect: () => Promise<void>;
  testConnection: () => Promise<boolean>;
  cancelConnection: () => Promise<void>;
  setConnectionState: (state: Partial<ConnectionState>) => void;
}

export const useConnectionStore = create<ConnectionStoreState>((set) => ({
  isConnected: false,
  isConnecting: false,
  currentProfile: undefined,
  connectionMessage: undefined,

  connectWithProfile: async (profileId: string) => {
    set({ isConnecting: true });
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
        isConnecting: false,
        currentProfile: profile,
        connectionMessage: message
      });
    } catch (error) {
      set({
        isConnected: false,
        isConnecting: false,
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

  cancelConnection: async () => {
    try {
      await invoke<string>('cancel_connection');
      set({ isConnecting: false });
    } catch (error) {
      console.error('Failed to cancel connection:', error);
    }
  },

  setConnectionState: (state: Partial<ConnectionState>) => {
    set(state);
  }
}));