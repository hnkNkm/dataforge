import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  ConnectionProfile,
  CreateProfileRequest,
  UpdateProfileRequest
} from '../types/profile';

interface ProfileState {
  profiles: ConnectionProfile[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadProfiles: () => Promise<void>;
  createProfile: (request: CreateProfileRequest) => Promise<ConnectionProfile>;
  updateProfile: (request: UpdateProfileRequest) => Promise<ConnectionProfile>;
  deleteProfile: (id: string) => Promise<void>;
  getProfile: (id: string) => Promise<ConnectionProfile>;
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  profiles: [],
  isLoading: false,
  error: null,

  loadProfiles: async () => {
    set({ isLoading: true, error: null });
    try {
      const profiles = await invoke<ConnectionProfile[]>('list_profiles');
      set({ profiles, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : String(error),
        isLoading: false
      });
      throw error;
    }
  },

  createProfile: async (request: CreateProfileRequest) => {
    set({ isLoading: true, error: null });
    try {
      const newProfile = await invoke<ConnectionProfile>('create_profile', { request });
      set((state) => ({
        profiles: [...state.profiles, newProfile],
        isLoading: false
      }));
      return newProfile;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : String(error),
        isLoading: false
      });
      throw error;
    }
  },

  updateProfile: async (request: UpdateProfileRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updatedProfile = await invoke<ConnectionProfile>('update_profile', { request });
      set((state) => ({
        profiles: state.profiles.map(p =>
          p.id === updatedProfile.id ? updatedProfile : p
        ),
        isLoading: false
      }));
      return updatedProfile;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : String(error),
        isLoading: false
      });
      throw error;
    }
  },

  deleteProfile: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('delete_profile', { id });
      set((state) => ({
        profiles: state.profiles.filter(p => p.id !== id),
        isLoading: false
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : String(error),
        isLoading: false
      });
      throw error;
    }
  },

  getProfile: async (id: string) => {
    const { profiles } = get();
    const cachedProfile = profiles.find(p => p.id === id);
    if (cachedProfile) return cachedProfile;

    try {
      const profile = await invoke<ConnectionProfile>('get_profile', { id });
      return profile;
    } catch (error) {
      throw error;
    }
  }
}));