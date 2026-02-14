import { create } from "zustand";
import * as api from "../api/tox";
import type { FriendInfo, FriendRequest } from "../api/tox";

interface FriendState {
  friends: FriendInfo[];
  friendRequests: FriendRequest[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadFriends: () => Promise<void>;
  loadFriendRequests: () => Promise<void>;
  addFriend: (toxId: string, message: string) => Promise<void>;
  acceptRequest: (publicKey: string) => Promise<void>;
  denyRequest: (publicKey: string) => Promise<void>;
  removeFriend: (friendNumber: number) => Promise<void>;

  // Event-driven updates
  updateFriendName: (friendNumber: number, name: string) => void;
  updateFriendStatusMessage: (friendNumber: number, message: string) => void;
  updateFriendStatus: (friendNumber: number, status: string) => void;
  updateFriendConnectionStatus: (friendNumber: number, connected: boolean, status: string) => void;
  addIncomingRequest: (publicKey: string, message: string) => void;

  clearError: () => void;
}

export const useFriendStore = create<FriendState>((set) => ({
  friends: [],
  friendRequests: [],
  isLoading: false,
  error: null,

  loadFriends: async () => {
    try {
      const friends = await api.getFriends();
      set({ friends });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadFriendRequests: async () => {
    try {
      const friendRequests = await api.getFriendRequests();
      set({ friendRequests });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  addFriend: async (toxId, message) => {
    set({ isLoading: true, error: null });
    try {
      await api.addFriend(toxId, message);
      // Reload friend list after adding
      const friends = await api.getFriends();
      set({ friends, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  acceptRequest: async (publicKey) => {
    set({ isLoading: true, error: null });
    try {
      await api.acceptFriendRequest(publicKey);
      // Reload both lists
      const [friends, friendRequests] = await Promise.all([
        api.getFriends(),
        api.getFriendRequests(),
      ]);
      set({ friends, friendRequests, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  denyRequest: async (publicKey) => {
    try {
      await api.denyFriendRequest(publicKey);
      set((s) => ({
        friendRequests: s.friendRequests.filter((r) => r.public_key !== publicKey),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  removeFriend: async (friendNumber) => {
    try {
      await api.removeFriend(friendNumber);
      set((s) => ({
        friends: s.friends.filter((f) => f.friend_number !== friendNumber),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateFriendName: (friendNumber, name) => {
    set((s) => ({
      friends: s.friends.map((f) =>
        f.friend_number === friendNumber ? { ...f, name } : f,
      ),
    }));
  },

  updateFriendStatusMessage: (friendNumber, message) => {
    set((s) => ({
      friends: s.friends.map((f) =>
        f.friend_number === friendNumber ? { ...f, status_message: message } : f,
      ),
    }));
  },

  updateFriendStatus: (friendNumber, status) => {
    set((s) => ({
      friends: s.friends.map((f) =>
        f.friend_number === friendNumber
          ? { ...f, user_status: status as FriendInfo["user_status"] }
          : f,
      ),
    }));
  },

  updateFriendConnectionStatus: (friendNumber, connected, status) => {
    set((s) => ({
      friends: s.friends.map((f) =>
        f.friend_number === friendNumber
          ? {
              ...f,
              connection_status: status as FriendInfo["connection_status"],
              last_seen: !connected ? new Date().toISOString() : f.last_seen,
            }
          : f,
      ),
    }));
  },

  addIncomingRequest: (publicKey, message) => {
    set((s) => {
      // Avoid duplicates
      if (s.friendRequests.some((r) => r.public_key === publicKey)) {
        return s;
      }
      return {
        friendRequests: [
          {
            public_key: publicKey,
            message,
            received_at: new Date().toISOString(),
          },
          ...s.friendRequests,
        ],
      };
    });
  },

  clearError: () => set({ error: null }),
}));
