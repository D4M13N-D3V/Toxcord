import { create } from "zustand";
import * as api from "../api/tox";

interface AuthState {
  isLoggedIn: boolean;
  profileName: string | null;
  toxId: string | null;
  displayName: string | null;
  statusMessage: string | null;
  isConnected: boolean;
  connectionType: string;
  isLoading: boolean;
  error: string | null;
  profiles: string[];

  loadProfiles: () => Promise<void>;
  createProfile: (profileName: string, password: string, displayName: string) => Promise<void>;
  loadProfile: (profileName: string, password: string) => Promise<void>;
  deleteProfile: (profileName: string) => Promise<void>;
  logout: () => Promise<void>;
  setConnectionStatus: (connected: boolean, status: string) => void;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  isLoggedIn: false,
  profileName: null,
  toxId: null,
  displayName: null,
  statusMessage: null,
  isConnected: false,
  connectionType: "none",
  isLoading: false,
  error: null,
  profiles: [],

  loadProfiles: async () => {
    try {
      const profiles = await api.listProfiles();
      set({ profiles });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createProfile: async (profileName, password, displayName) => {
    set({ isLoading: true, error: null });
    try {
      const info = await api.createProfile(profileName, password, displayName);
      set({
        isLoggedIn: true,
        profileName,
        toxId: info.tox_id,
        displayName: info.name,
        statusMessage: info.status_message,
        isLoading: false,
      });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  loadProfile: async (profileName, password) => {
    set({ isLoading: true, error: null });
    try {
      const info = await api.loadProfile(profileName, password);
      set({
        isLoggedIn: true,
        profileName,
        toxId: info.tox_id,
        displayName: info.name,
        statusMessage: info.status_message,
        isLoading: false,
      });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  deleteProfile: async (profileName) => {
    set({ isLoading: true, error: null });
    try {
      await api.deleteProfile(profileName);
      // Refresh the profiles list
      const profiles = await api.listProfiles();
      set({ profiles, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  logout: async () => {
    try {
      await api.logout();
    } catch (_e) {
      // Ignore errors during logout
    }
    set({
      isLoggedIn: false,
      profileName: null,
      toxId: null,
      displayName: null,
      statusMessage: null,
      isConnected: false,
      connectionType: "none",
    });
  },

  setConnectionStatus: (connected, status) => {
    set({ isConnected: connected, connectionType: status });
  },

  clearError: () => set({ error: null }),
}));
