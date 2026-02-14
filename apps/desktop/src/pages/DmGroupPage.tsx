import { useEffect, useRef, useState, useCallback } from "react";
import { useNavigationStore } from "../stores/navigationStore";
import { useGuildStore } from "../stores/guildStore";
import { useChannelMessageStore } from "../stores/channelMessageStore";
import type { ChannelMessage } from "../api/tox";

const EMPTY_MESSAGES: never[] = [];
const EMPTY_CHANNELS: never[] = [];
const EMPTY_MEMBERS: never[] = [];

export function DmGroupPage() {
  const selectedDmGroupId = useNavigationStore((s) => s.selectedDmGroupId);

  if (!selectedDmGroupId) {
    return (
      <div className="flex flex-1 items-center justify-center bg-discord-chat">
        <p className="text-discord-muted">Select a group to start messaging</p>
      </div>
    );
  }

  return <DmGroupConversation dmGroupId={selectedDmGroupId} />;
}

function DmGroupConversation({ dmGroupId }: { dmGroupId: string }) {
  const dmGroups = useGuildStore((s) => s.dmGroups);
  const channels = useGuildStore((s) => s.channels[dmGroupId] ?? EMPTY_CHANNELS);
  const loadChannels = useGuildStore((s) => s.loadChannels);
  const loadMembers = useGuildStore((s) => s.loadMembers);
  const members = useGuildStore((s) => s.members[dmGroupId] ?? EMPTY_MEMBERS);

  const dmGroup = dmGroups.find((g) => g.id === dmGroupId);
  const groupName = dmGroup?.name ?? "Group";

  // DM groups have a single channel for messages
  const channelId = channels[0]?.id;

  useEffect(() => {
    loadChannels(dmGroupId);
    loadMembers(dmGroupId);
  }, [dmGroupId, loadChannels, loadMembers]);

  if (!channelId) {
    return (
      <div className="flex flex-1 items-center justify-center bg-discord-chat">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-discord-blurple border-t-transparent" />
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col bg-discord-chat">
      <DmGroupHeader name={groupName} memberCount={members.length} />
      <MessageArea channelId={channelId} groupName={groupName} />
      <MessageInput dmGroupId={dmGroupId} channelId={channelId} groupName={groupName} />
    </div>
  );
}

function DmGroupHeader({ name, memberCount }: { name: string; memberCount: number }) {
  const [showMenu, setShowMenu] = useState(false);
  const selectedDmGroupId = useNavigationStore((s) => s.selectedDmGroupId);
  const setPage = useNavigationStore((s) => s.setPage);
  const leaveGuild = useGuildStore((s) => s.leaveGuild);

  const handleLeave = async () => {
    if (selectedDmGroupId && confirm("Are you sure you want to leave this group?")) {
      await leaveGuild(selectedDmGroupId);
      setPage("friends");
    }
  };

  return (
    <div className="flex h-12 items-center justify-between px-4">
      <div className="flex items-center gap-2">
        <div className="relative">
          <div className="flex h-6 w-6 items-center justify-center rounded-full bg-discord-green">
            <svg className="h-3.5 w-3.5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
            </svg>
          </div>
        </div>
        <h3 className="font-semibold text-white">{name}</h3>
        <div className="h-4 w-px bg-discord-input mx-1" />
        <span className="text-sm text-discord-muted">{memberCount} members</span>
      </div>

      <div className="relative">
        <button
          onClick={() => setShowMenu(!showMenu)}
          className="rounded p-1 text-discord-muted hover:bg-discord-hover hover:text-white"
        >
          <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
            <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
          </svg>
        </button>

        {showMenu && (
          <>
            <div
              className="fixed inset-0 z-40"
              onClick={() => setShowMenu(false)}
            />
            <div className="absolute right-0 top-8 z-50 w-48 rounded-md bg-discord-darker py-1 shadow-lg">
              <button
                onClick={handleLeave}
                className="flex w-full items-center px-3 py-2 text-sm text-discord-red hover:bg-discord-hover"
              >
                <svg
                  className="mr-2 h-4 w-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  strokeWidth={2}
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                  />
                </svg>
                Leave Group
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function MessageArea({ channelId, groupName }: { channelId: string; groupName: string }) {
  const messages = useChannelMessageStore((s) => s.messages[channelId] ?? EMPTY_MESSAGES);
  const isLoading = useChannelMessageStore((s) => s.isLoading);
  const hasMore = useChannelMessageStore((s) => s.hasMore[channelId] ?? true);
  const loadMessages = useChannelMessageStore((s) => s.loadMessages);

  const parentRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);

  useEffect(() => {
    loadMessages(channelId);
  }, [channelId, loadMessages]);

  useEffect(() => {
    if (isAtBottom && parentRef.current) {
      parentRef.current.scrollTop = parentRef.current.scrollHeight;
    }
  }, [messages.length, isAtBottom]);

  const handleScroll = useCallback(() => {
    if (!parentRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = parentRef.current;
    setIsAtBottom(scrollHeight - scrollTop - clientHeight < 50);

    if (scrollTop < 100 && hasMore && !isLoading && messages.length > 0) {
      const oldestTimestamp = messages[0]?.timestamp;
      if (oldestTimestamp) {
        const prevHeight = scrollHeight;
        loadMessages(channelId, oldestTimestamp).then(() => {
          if (parentRef.current) {
            const newHeight = parentRef.current.scrollHeight;
            parentRef.current.scrollTop = newHeight - prevHeight;
          }
        });
      }
    }
  }, [hasMore, isLoading, messages, channelId, loadMessages]);

  const groupedMessages = groupMessages(messages);

  if (messages.length === 0 && !isLoading) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center text-discord-muted">
        <div className="mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-discord-green">
          <svg className="h-10 w-10 text-white" fill="currentColor" viewBox="0 0 24 24">
            <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
          </svg>
        </div>
        <h3 className="mb-1 text-xl font-bold text-white">{groupName}</h3>
        <p className="text-sm">
          This is the beginning of your group chat.
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
            showDateSeparator={
              groupIdx === 0 ||
              !isSameDay(group.messages[0].timestamp, groupedMessages[groupIdx - 1].messages[0].timestamp)
            }
          />
        ))}
      </div>
    </div>
  );
}

interface MessageGroupData {
  senderName: string;
  senderPk: string;
  isOwn: boolean;
  messages: ChannelMessage[];
}

function MessageGroup({
  group,
  showDateSeparator,
}: {
  group: MessageGroupData;
  showDateSeparator: boolean;
}) {
  const firstMsg = group.messages[0];

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
        <div className="mr-4 mt-0.5 flex-shrink-0">
          <div className={`flex h-10 w-10 items-center justify-center rounded-full text-sm font-bold text-white ${
            group.isOwn ? "bg-discord-green" : "bg-discord-blurple"
          }`}>
            {group.senderName[0]?.toUpperCase() ?? "?"}
          </div>
        </div>

        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <span className={`text-sm font-medium ${group.isOwn ? "text-discord-green" : "text-white"}`}>
              {group.senderName}
            </span>
            <span className="text-xs text-discord-muted">
              {formatTime(firstMsg.timestamp)}
            </span>
          </div>

          {group.messages.map((msg) => (
            <div key={msg.id} className="group/msg relative">
              {msg.message_type === "action" ? (
                <p className="text-sm italic text-discord-muted">
                  * {group.senderName} {msg.content}
                </p>
              ) : (
                <p className="text-sm text-discord-text leading-[1.375rem]">{msg.content}</p>
              )}
            </div>
          ))}
        </div>
      </div>
    </>
  );
}

function MessageInput({
  dmGroupId,
  channelId,
  groupName,
}: {
  dmGroupId: string;
  channelId: string;
  groupName: string;
}) {
  const [content, setContent] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const sendDmGroupMessage = useChannelMessageStore((s) => s.sendDmGroupMessage);

  const handleSubmit = async () => {
    const trimmed = content.trim();
    if (!trimmed) return;

    setContent("");
    await sendDmGroupMessage(dmGroupId, channelId, trimmed);

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
              handleInput();
            }}
            onKeyDown={handleKeyDown}
            placeholder={`Message ${groupName}`}
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

function groupMessages(messages: ChannelMessage[]): MessageGroupData[] {
  const groups: MessageGroupData[] = [];

  for (const msg of messages) {
    const lastGroup = groups[groups.length - 1];

    if (
      lastGroup &&
      lastGroup.senderPk === msg.sender_public_key &&
      timeDiffMinutes(lastGroup.messages[lastGroup.messages.length - 1].timestamp, msg.timestamp) < 7
    ) {
      lastGroup.messages.push(msg);
    } else {
      groups.push({
        senderName: msg.sender_name || "Unknown",
        senderPk: msg.sender_public_key,
        isOwn: msg.is_own,
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
