import { useEffect, useRef, useState, useCallback, useMemo } from "react";
import { useMessageStore } from "../stores/messageStore";
import { useFriendStore } from "../stores/friendStore";
import { useNavigationStore } from "../stores/navigationStore";
import { useCallStore } from "../stores/callStore";
import * as api from "../api/tox";
import type { DirectMessage } from "../api/tox";
import { MiniCallIndicator } from "../components/call/VoiceCallUI";
import { CallPanel } from "../components/call/CallPanel";
import { FullscreenVideoModal } from "../components/video/FullscreenVideoModal";

const EMPTY_MESSAGES: never[] = [];

export function DMPage() {
  const selectedFriendNumber = useNavigationStore((s) => s.selectedFriendNumber);

  if (selectedFriendNumber === null) {
    return (
      <div className="flex flex-1 items-center justify-center bg-discord-chat">
        <p className="text-discord-muted">Select a conversation to start messaging</p>
      </div>
    );
  }

  return <DMConversation friendNumber={selectedFriendNumber} />;
}

function DMConversation({ friendNumber }: { friendNumber: number }) {
  const friends = useFriendStore((s) => s.friends);
  const activeCall = useCallStore((s) => s.activeCall);
  const friend = useMemo(
    () => friends.find((f) => f.friend_number === friendNumber),
    [friends, friendNumber],
  );
  const friendName = friend?.name || (friend?.public_key ? friend.public_key.slice(0, 16) + "..." : "Unknown");
  const isOnline = friend ? friend.connection_status !== "none" : false;

  // Check if we're in an active call with this friend
  const isInActiveCall = activeCall?.friendNumber === friendNumber && activeCall?.status === "in_progress";

  return (
    <>
      <div className="flex flex-1 flex-col bg-discord-chat">
        <DMHeader
          friendNumber={friendNumber}
          name={friendName}
          isOnline={isOnline}
          statusMessage={friend?.status_message}
        />
        {/* Show call panel above chat when in a call */}
        {isInActiveCall && <CallPanel />}
        {/* Chat is always visible */}
        <MessageArea friendNumber={friendNumber} friendName={friendName} />
        <MessageInput friendNumber={friendNumber} friendName={friendName} isOnline={isOnline} />
      </div>
      {/* Fullscreen video modal */}
      <FullscreenVideoModal />
    </>
  );
}

function DMHeader({
  friendNumber,
  name,
  isOnline,
  statusMessage,
}: {
  friendNumber: number;
  name: string;
  isOnline: boolean;
  statusMessage?: string;
}) {
  const activeCall = useCallStore((s) => s.activeCall);
  const startCall = useCallStore((s) => s.startCall);
  const isInCallWithFriend = activeCall?.friendNumber === friendNumber;

  return (
    <div className="flex h-12 items-center justify-between px-4">
      <div className="flex items-center gap-2">
        <div className="relative">
          <div className="flex h-6 w-6 items-center justify-center rounded-full bg-discord-blurple text-xs font-bold text-white">
            {name[0]?.toUpperCase() ?? "?"}
          </div>
          <div
            className={`absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-discord-chat ${
              isOnline ? "bg-discord-green" : "bg-discord-muted"
            }`}
          />
        </div>
        <h3 className="font-semibold text-white">{name}</h3>
        {statusMessage && (
          <>
            <div className="h-4 w-px bg-discord-input mx-1" />
            <span className="text-sm text-discord-muted">{statusMessage}</span>
          </>
        )}
      </div>

      {/* Call controls */}
      <div className="flex items-center gap-2">
        {isInCallWithFriend ? (
          <MiniCallIndicator />
        ) : (
          <>
            {/* Voice call button */}
            <button
              onClick={() => startCall(friendNumber, name, false)}
              disabled={!isOnline}
              className={`flex h-8 w-8 items-center justify-center rounded-md transition-colors ${
                isOnline
                  ? "text-discord-muted hover:bg-discord-hover hover:text-white"
                  : "cursor-not-allowed text-discord-muted/50"
              }`}
              title={isOnline ? "Start voice call" : "Friend is offline"}
            >
              <PhoneIcon className="h-5 w-5" />
            </button>
            {/* Video call button */}
            <button
              onClick={() => startCall(friendNumber, name, true)}
              disabled={!isOnline}
              className={`flex h-8 w-8 items-center justify-center rounded-md transition-colors ${
                isOnline
                  ? "text-discord-muted hover:bg-discord-hover hover:text-white"
                  : "cursor-not-allowed text-discord-muted/50"
              }`}
              title={isOnline ? "Start video call" : "Friend is offline"}
            >
              <VideoIcon className="h-5 w-5" />
            </button>
          </>
        )}
      </div>
    </div>
  );
}

// Icons for call buttons
function PhoneIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M20.01 15.38c-1.23 0-2.42-.2-3.53-.56-.35-.12-.74-.03-1.01.24l-1.57 1.97c-2.83-1.35-5.48-3.9-6.89-6.83l1.95-1.66c.27-.28.35-.67.24-1.02-.37-1.11-.56-2.3-.56-3.53 0-.54-.45-.99-.99-.99H4.19C3.65 3 3 3.24 3 3.99 3 13.28 10.73 21 20.01 21c.71 0 .99-.63.99-1.18v-3.45c0-.54-.45-.99-.99-.99z" />
    </svg>
  );
}

function VideoIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z" />
    </svg>
  );
}

function MessageArea({ friendNumber, friendName }: { friendNumber: number; friendName: string }) {
  const messages = useMessageStore((s) => s.conversations[friendNumber] ?? EMPTY_MESSAGES);
  const isLoading = useMessageStore((s) => s.isLoading);
  const hasMore = useMessageStore((s) => s.hasMore[friendNumber] ?? true);
  const loadMessages = useMessageStore((s) => s.loadMessages);
  const markRead = useMessageStore((s) => s.markRead);
  const typing = useMessageStore((s) => s.typing[friendNumber] ?? false);

  const parentRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const prevMessageCount = useRef(0);

  // Load initial messages
  useEffect(() => {
    loadMessages(friendNumber);
    markRead(friendNumber);
  }, [friendNumber, loadMessages, markRead]);

  // Mark as read when messages arrive and we're viewing this conversation
  useEffect(() => {
    if (messages.length > prevMessageCount.current) {
      markRead(friendNumber);
    }
    prevMessageCount.current = messages.length;
  }, [messages.length, friendNumber, markRead]);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (isAtBottom && parentRef.current) {
      parentRef.current.scrollTop = parentRef.current.scrollHeight;
    }
  }, [messages.length, isAtBottom]);

  const handleScroll = useCallback(() => {
    if (!parentRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = parentRef.current;
    setIsAtBottom(scrollHeight - scrollTop - clientHeight < 50);

    // Load more when scrolled to top
    if (scrollTop < 100 && hasMore && !isLoading && messages.length > 0) {
      const oldestTimestamp = messages[0]?.timestamp;
      if (oldestTimestamp) {
        const prevHeight = scrollHeight;
        loadMessages(friendNumber, oldestTimestamp).then(() => {
          // Maintain scroll position after loading older messages
          if (parentRef.current) {
            const newHeight = parentRef.current.scrollHeight;
            parentRef.current.scrollTop = newHeight - prevHeight;
          }
        });
      }
    }
  }, [hasMore, isLoading, messages, friendNumber, loadMessages]);

  // Group messages by sender and time proximity
  const groupedMessages = groupMessages(messages);

  if (messages.length === 0 && !isLoading) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center text-discord-muted">
        <div className="mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-discord-blurple">
          <span className="text-3xl font-bold text-white">{friendName[0]?.toUpperCase()}</span>
        </div>
        <h3 className="mb-1 text-xl font-bold text-white">{friendName}</h3>
        <p className="text-sm">
          This is the beginning of your direct message history with <strong className="text-white">{friendName}</strong>.
        </p>
      </div>
    );
  }

  return (
    <div
      ref={parentRef}
      className="flex-1 overflow-y-auto"
      onScroll={handleScroll}
    >
      {/* Loading indicator for older messages */}
      {isLoading && (
        <div className="flex justify-center py-4">
          <div className="h-5 w-5 animate-spin rounded-full border-2 border-discord-blurple border-t-transparent" />
        </div>
      )}

      <div className="flex flex-col pb-4">
        {groupedMessages.map((group, groupIdx) => (
          <MessageGroup
            key={group.messages[0].id}
            group={group}
            friendName={friendName}
            showDateSeparator={
              groupIdx === 0 ||
              !isSameDay(group.messages[0].timestamp, groupedMessages[groupIdx - 1].messages[0].timestamp)
            }
          />
        ))}
      </div>

      {/* Typing indicator */}
      {typing && (
        <div className="px-4 pb-3">
          <div className="flex items-center gap-2 text-sm text-discord-muted">
            <div className="flex gap-0.5">
              <span className="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-discord-muted" style={{ animationDelay: "0ms" }} />
              <span className="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-discord-muted" style={{ animationDelay: "150ms" }} />
              <span className="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-discord-muted" style={{ animationDelay: "300ms" }} />
            </div>
            <span>{friendName} is typing...</span>
          </div>
        </div>
      )}
    </div>
  );
}

interface MessageGroupData {
  sender: string;
  isOutgoing: boolean;
  messages: DirectMessage[];
}

function MessageGroup({
  group,
  friendName,
  showDateSeparator,
}: {
  group: MessageGroupData;
  friendName: string;
  showDateSeparator: boolean;
}) {
  const firstMsg = group.messages[0];
  const senderName = group.isOutgoing ? "You" : friendName;

  return (
    <>
      {showDateSeparator && (
        <div className="my-2 flex items-center px-4">
          <div className="flex-1 border-t border-discord-hover" />
          <span className="px-2 text-xs font-semibold text-discord-muted">
            {formatDate(firstMsg.timestamp)}
          </span>
          <div className="flex-1 border-t border-discord-hover" />
        </div>
      )}

      <div className="group mt-[1.0625rem] flex px-4 py-0.5 hover:bg-discord-hover/30">
        {/* Avatar */}
        <div className="mr-4 mt-0.5 flex-shrink-0">
          <div className={`flex h-10 w-10 items-center justify-center rounded-full text-sm font-bold text-white ${
            group.isOutgoing ? "bg-discord-green" : "bg-discord-blurple"
          }`}>
            {senderName[0]?.toUpperCase()}
          </div>
        </div>

        {/* Content */}
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <span className={`text-sm font-medium ${group.isOutgoing ? "text-discord-green" : "text-white"}`}>
              {senderName}
            </span>
            <span className="text-xs text-discord-muted">
              {formatTime(firstMsg.timestamp)}
            </span>
          </div>

          {group.messages.map((msg) => (
            <div key={msg.id} className="group/msg relative">
              {msg.message_type === "action" ? (
                <p className="text-sm italic text-discord-muted">
                  * {senderName} {msg.content}
                </p>
              ) : (
                <p className="text-sm text-discord-text leading-[1.375rem]">{msg.content}</p>
              )}
              {msg.is_outgoing && !msg.delivered && (
                <span className="ml-1 text-xs text-discord-muted">(queued)</span>
              )}
            </div>
          ))}
        </div>
      </div>
    </>
  );
}

function MessageInput({
  friendNumber,
  friendName,
  isOnline,
}: {
  friendNumber: number;
  friendName: string;
  isOnline: boolean;
}) {
  const [content, setContent] = useState("");
  const sendMessage = useMessageStore((s) => s.sendMessage);
  const typingTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Focus input when friend changes
  useEffect(() => {
    textareaRef.current?.focus();
  }, [friendNumber]);

  const handleTyping = useCallback(() => {
    if (!isTypingRef.current) {
      isTypingRef.current = true;
      api.setTyping(friendNumber, true).catch(() => {});
    }

    // Reset the timeout
    if (typingTimeout.current) {
      clearTimeout(typingTimeout.current);
    }
    typingTimeout.current = setTimeout(() => {
      isTypingRef.current = false;
      api.setTyping(friendNumber, false).catch(() => {});
    }, 3000);
  }, [friendNumber]);

  const stopTyping = useCallback(() => {
    if (isTypingRef.current) {
      isTypingRef.current = false;
      api.setTyping(friendNumber, false).catch(() => {});
    }
    if (typingTimeout.current) {
      clearTimeout(typingTimeout.current);
      typingTimeout.current = null;
    }
  }, [friendNumber]);

  const handleSubmit = async () => {
    const trimmed = content.trim();
    if (!trimmed) return;

    setContent("");
    stopTyping();
    await sendMessage(friendNumber, trimmed);

    // Re-focus and auto-resize
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  // Auto-resize textarea
  const handleInput = () => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 200) + "px";
    }
  };

  return (
    <div className="px-4 pb-6 pt-0">
      <div className="rounded-lg bg-discord-input">
        <div className="flex items-end">
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => {
              setContent(e.target.value);
              handleTyping();
              handleInput();
            }}
            onKeyDown={handleKeyDown}
            placeholder={
              isOnline
                ? `Message @${friendName}`
                : `@${friendName} is offline — message will be queued`
            }
            className="flex-1 resize-none bg-transparent px-4 py-3 text-sm text-white placeholder-discord-muted outline-none"
            rows={1}
            style={{ maxHeight: "200px" }}
          />
        </div>
      </div>
    </div>
  );
}

// ─── Helpers ──────────────────────────────────────────────────────────

function groupMessages(messages: DirectMessage[]): MessageGroupData[] {
  const groups: MessageGroupData[] = [];

  for (const msg of messages) {
    const lastGroup = groups[groups.length - 1];

    // Group if same sender and within 7 minutes
    if (
      lastGroup &&
      lastGroup.sender === msg.sender &&
      lastGroup.isOutgoing === msg.is_outgoing &&
      timeDiffMinutes(lastGroup.messages[lastGroup.messages.length - 1].timestamp, msg.timestamp) < 7
    ) {
      lastGroup.messages.push(msg);
    } else {
      groups.push({
        sender: msg.sender,
        isOutgoing: msg.is_outgoing,
        messages: [msg],
      });
    }
  }

  return groups;
}

function timeDiffMinutes(a: string, b: string): number {
  return Math.abs(new Date(b).getTime() - new Date(a).getTime()) / 60000;
}

function isSameDay(a: string, b: string): boolean {
  const da = new Date(a);
  const db = new Date(b);
  return (
    da.getFullYear() === db.getFullYear() &&
    da.getMonth() === db.getMonth() &&
    da.getDate() === db.getDate()
  );
}

function formatDate(iso: string): string {
  const date = new Date(iso);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / 86400000);

  if (days === 0) return "Today";
  if (days === 1) return "Yesterday";
  return date.toLocaleDateString(undefined, {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });
}
