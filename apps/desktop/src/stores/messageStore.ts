import { create } from "zustand";
import * as api from "../api/tox";
import type { DirectMessage } from "../api/tox";

interface MessageState {
  /** Messages keyed by friend_number */
  conversations: Record<number, DirectMessage[]>;
  /** Typing indicators keyed by friend_number */
  typing: Record<number, boolean>;
  /** Unread counts keyed by friend_number */
  unreadCounts: Record<number, number>;
  /** Whether we're currently loading messages */
  isLoading: boolean;
  /** Whether there are more messages to load (for scroll-to-load-more) */
  hasMore: Record<number, boolean>;

  // Actions
  loadMessages: (friendNumber: number, beforeTimestamp?: string) => Promise<void>;
  sendMessage: (friendNumber: number, content: string) => Promise<void>;
  addIncomingMessage: (msg: DirectMessage) => void;
  setFriendTyping: (friendNumber: number, isTyping: boolean) => void;
  markRead: (friendNumber: number) => Promise<void>;
  clearConversation: (friendNumber: number) => void;
}

const PAGE_SIZE = 50;

export const useMessageStore = create<MessageState>((set) => ({
  conversations: {},
  typing: {},
  unreadCounts: {},
  isLoading: false,
  hasMore: {},

  loadMessages: async (friendNumber, beforeTimestamp) => {
    set({ isLoading: true });
    try {
      const messages = await api.getDirectMessages(
        friendNumber,
        PAGE_SIZE,
        beforeTimestamp,
      );

      set((s) => {
        const existing = beforeTimestamp ? (s.conversations[friendNumber] ?? []) : [];
        // Messages come from DB in DESC order, reverse to chronological
        const reversed = [...messages].reverse();
        const merged = [...reversed, ...existing];

        return {
          conversations: { ...s.conversations, [friendNumber]: merged },
          hasMore: { ...s.hasMore, [friendNumber]: messages.length === PAGE_SIZE },
          isLoading: false,
        };
      });
    } catch {
      set({ isLoading: false });
    }
  },

  sendMessage: async (friendNumber, content) => {
    try {
      const result = await api.sendDirectMessage(friendNumber, content);

      // Add the sent message to the conversation immediately
      const msg: DirectMessage = {
        id: result.id,
        friend_number: friendNumber,
        sender: "self",
        content,
        message_type: "normal",
        timestamp: result.timestamp,
        is_outgoing: true,
        delivered: result.delivered,
        read: false,
      };

      set((s) => ({
        conversations: {
          ...s.conversations,
          [friendNumber]: [...(s.conversations[friendNumber] ?? []), msg],
        },
      }));
    } catch (e) {
      console.error("Failed to send message:", e);
    }
  },

  addIncomingMessage: (msg) => {
    set((s) => {
      const existing = s.conversations[msg.friend_number] ?? [];
      // Avoid duplicates
      if (existing.some((m) => m.id === msg.id)) return s;

      const currentUnread = s.unreadCounts[msg.friend_number] ?? 0;

      return {
        conversations: {
          ...s.conversations,
          [msg.friend_number]: [...existing, msg],
        },
        unreadCounts: {
          ...s.unreadCounts,
          [msg.friend_number]: currentUnread + 1,
        },
      };
    });
  },

  setFriendTyping: (friendNumber, isTyping) => {
    set((s) => ({
      typing: { ...s.typing, [friendNumber]: isTyping },
    }));
  },

  markRead: async (friendNumber) => {
    try {
      await api.markMessagesRead(friendNumber);
      set((s) => ({
        unreadCounts: { ...s.unreadCounts, [friendNumber]: 0 },
        conversations: {
          ...s.conversations,
          [friendNumber]: (s.conversations[friendNumber] ?? []).map((m) =>
            !m.is_outgoing && !m.read ? { ...m, read: true } : m,
          ),
        },
      }));
    } catch {
      // ignore
    }
  },

  clearConversation: (friendNumber) => {
    set((s) => {
      const { [friendNumber]: _, ...rest } = s.conversations;
      return { conversations: rest };
    });
  },
}));
