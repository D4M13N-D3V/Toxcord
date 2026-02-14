import { create } from "zustand";
import * as api from "../api/tox";
import type { ChannelMessage } from "../api/tox";

interface ChannelMessageState {
  messages: Record<string, ChannelMessage[]>;
  isLoading: boolean;
  hasMore: Record<string, boolean>;

  loadMessages: (channelId: string, beforeTimestamp?: string) => Promise<void>;
  sendMessage: (guildId: string, channelId: string, content: string) => Promise<void>;
  sendDmGroupMessage: (dmGroupId: string, channelId: string, content: string) => Promise<void>;
  addIncomingMessage: (channelId: string, msg: ChannelMessage) => void;
  clearChannel: (channelId: string) => void;
}

const PAGE_SIZE = 50;

export const useChannelMessageStore = create<ChannelMessageState>((set) => ({
  messages: {},
  isLoading: false,
  hasMore: {},

  loadMessages: async (channelId, beforeTimestamp) => {
    set({ isLoading: true });
    try {
      const messages = await api.getChannelMessages(
        channelId,
        PAGE_SIZE,
        beforeTimestamp,
      );

      set((s) => {
        const existing = beforeTimestamp ? (s.messages[channelId] ?? []) : [];
        const reversed = [...messages].reverse();
        const merged = [...reversed, ...existing];

        return {
          messages: { ...s.messages, [channelId]: merged },
          hasMore: { ...s.hasMore, [channelId]: messages.length === PAGE_SIZE },
          isLoading: false,
        };
      });
    } catch {
      set({ isLoading: false });
    }
  },

  sendMessage: async (guildId, channelId, content) => {
    try {
      const msg = await api.sendChannelMessage(guildId, channelId, content);

      set((s) => ({
        messages: {
          ...s.messages,
          [channelId]: [...(s.messages[channelId] ?? []), msg],
        },
      }));
    } catch (e) {
      console.error("Failed to send channel message:", e);
    }
  },

  sendDmGroupMessage: async (dmGroupId, channelId, content) => {
    try {
      const msg = await api.sendDmGroupMessage(dmGroupId, content);

      set((s) => ({
        messages: {
          ...s.messages,
          [channelId]: [...(s.messages[channelId] ?? []), msg],
        },
      }));
    } catch (e) {
      console.error("Failed to send DM group message:", e);
    }
  },

  addIncomingMessage: (channelId, msg) => {
    set((s) => {
      const existing = s.messages[channelId] ?? [];
      if (existing.some((m) => m.id === msg.id)) return s;

      return {
        messages: {
          ...s.messages,
          [channelId]: [...existing, msg],
        },
      };
    });
  },

  clearChannel: (channelId) => {
    set((s) => {
      const { [channelId]: _, ...rest } = s.messages;
      return { messages: rest };
    });
  },
}));
