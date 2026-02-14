import { useEffect, useState } from "react";
import { useFriendStore } from "../stores/friendStore";
import { useNavigationStore } from "../stores/navigationStore";
import { useGuildStore } from "../stores/guildStore";
import type { FriendInfo } from "../api/tox";

type Tab = "online" | "all" | "pending" | "add";

export function FriendsPage() {
  const [activeTab, setActiveTab] = useState<Tab>("online");
  const [showGroupModal, setShowGroupModal] = useState(false);
  const {
    friends,
    friendRequests,
    error,
    loadFriends,
    loadFriendRequests,
    clearError,
  } = useFriendStore();

  useEffect(() => {
    loadFriends();
    loadFriendRequests();
  }, [loadFriends, loadFriendRequests]);

  const onlineFriends = friends.filter((f) => f.connection_status !== "none");
  const pendingCount = friendRequests.length;

  return (
    <div className="flex flex-1 flex-col bg-discord-chat">
      {/* Header with tabs */}
      <div className="flex h-12 items-center px-4">
        <div className="mr-4 flex items-center gap-1 text-discord-muted">
          <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-1.053M18 8.625a4.5 4.5 0 11-9 0 4.5 4.5 0 019 0z" />
          </svg>
          <h3 className="font-semibold text-white">Friends</h3>
        </div>

        <div className="h-6 w-px bg-discord-input mx-2" />

        {/* Tab buttons */}
        <div className="flex gap-1">
          {(["online", "all", "pending"] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`rounded px-2 py-1 text-sm font-medium transition-colors ${
                activeTab === tab
                  ? "bg-discord-active text-white"
                  : "text-discord-muted hover:bg-discord-hover hover:text-white"
              }`}
            >
              {tab === "online" ? "Online" : tab === "all" ? "All" : "Pending"}
              {tab === "pending" && pendingCount > 0 && (
                <span className="ml-1 inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-discord-red px-1 text-xs font-bold text-white">
                  {pendingCount}
                </span>
              )}
            </button>
          ))}
          <button
            onClick={() => setActiveTab("add")}
            className={`rounded px-2 py-1 text-sm font-medium transition-colors ${
              activeTab === "add"
                ? "bg-discord-green text-white"
                : "bg-discord-green/20 text-discord-green hover:bg-discord-green/30"
            }`}
          >
            Add Friend
          </button>
        </div>

        <div className="ml-auto">
          <button
            onClick={() => setShowGroupModal(true)}
            className="flex items-center gap-1.5 rounded px-3 py-1 text-sm font-medium text-discord-muted transition-colors hover:bg-discord-hover hover:text-white"
            title="Create Group DM"
          >
            <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
            </svg>
            New Group
          </button>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="mx-4 mt-2 rounded bg-discord-red/20 p-3 text-sm text-discord-red">
          {error}
          <button onClick={clearError} className="ml-2 text-xs underline">
            dismiss
          </button>
        </div>
      )}

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto">
        {activeTab === "add" && <AddFriendPanel />}
        {activeTab === "pending" && <PendingPanel />}
        {activeTab === "online" && (
          <FriendListPanel
            friends={onlineFriends}
            emptyMessage="No friends are online right now"
            label={`Online — ${onlineFriends.length}`}
          />
        )}
        {activeTab === "all" && (
          <FriendListPanel
            friends={friends}
            emptyMessage="You haven't added any friends yet"
            label={`All Friends — ${friends.length}`}
          />
        )}
      </div>

      {showGroupModal && (
        <CreateGroupModal onClose={() => setShowGroupModal(false)} />
      )}
    </div>
  );
}

function AddFriendPanel() {
  const [toxId, setToxId] = useState("");
  const [message, setMessage] = useState("Hello! I'd like to add you on Toxcord.");
  const [success, setSuccess] = useState(false);
  const { addFriend, isLoading } = useFriendStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!toxId.trim()) return;
    await addFriend(toxId.trim(), message.trim());
    if (!useFriendStore.getState().error) {
      setToxId("");
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    }
  };

  return (
    <div className="p-4">
      <div className="mb-6">
        <h4 className="mb-1 text-sm font-bold uppercase text-white">Add Friend</h4>
        <p className="text-sm text-discord-muted">
          You can add friends by entering their Tox ID (76 characters).
        </p>
      </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="rounded-lg bg-discord-darker p-4">
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={toxId}
              onChange={(e) => setToxId(e.target.value)}
              placeholder="Enter a Tox ID"
              className="flex-1 rounded-md bg-discord-input p-2.5 text-sm text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
              autoFocus
            />
            <button
              type="submit"
              disabled={isLoading || toxId.trim().length < 64}
              className="rounded-md bg-discord-blurple px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {isLoading ? "Sending..." : "Send Friend Request"}
            </button>
          </div>

          {success && (
            <p className="mt-2 text-sm text-discord-green">
              Friend request sent successfully!
            </p>
          )}
        </div>

        <div>
          <label className="mb-1 block text-xs font-bold uppercase text-discord-muted">
            Message (optional)
          </label>
          <input
            type="text"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            className="w-full rounded-md bg-discord-darker p-2.5 text-sm text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
          />
        </div>
      </form>
    </div>
  );
}

function PendingPanel() {
  const { friendRequests, acceptRequest, denyRequest, isLoading } =
    useFriendStore();

  if (friendRequests.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-discord-muted">
        <p className="text-sm">No pending friend requests</p>
      </div>
    );
  }

  return (
    <div className="p-4">
      <h4 className="mb-3 text-xs font-bold uppercase text-discord-muted">
        Pending — {friendRequests.length}
      </h4>
      <div className="space-y-px">
        {friendRequests.map((req) => (
          <div
            key={req.public_key}
            className="flex items-center rounded-lg p-2.5 transition-colors hover:bg-discord-hover"
          >
            {/* Avatar */}
            <div className="mr-3 flex h-8 w-8 items-center justify-center rounded-full bg-discord-yellow text-sm font-bold text-white">
              ?
            </div>

            {/* Info */}
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium text-white">
                {req.public_key.slice(0, 16)}...
              </p>
              <p className="truncate text-xs text-discord-muted">
                {req.message || "No message"}
              </p>
            </div>

            {/* Actions */}
            <div className="flex gap-2">
              <button
                onClick={() => acceptRequest(req.public_key)}
                disabled={isLoading}
                className="flex h-9 w-9 items-center justify-center rounded-full bg-discord-darker text-discord-green transition-colors hover:bg-discord-green hover:text-white"
                title="Accept"
              >
                <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                </svg>
              </button>
              <button
                onClick={() => denyRequest(req.public_key)}
                disabled={isLoading}
                className="flex h-9 w-9 items-center justify-center rounded-full bg-discord-darker text-discord-red transition-colors hover:bg-discord-red hover:text-white"
                title="Deny"
              >
                <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function FriendListPanel({
  friends,
  emptyMessage,
  label,
}: {
  friends: FriendInfo[];
  emptyMessage: string;
  label: string;
}) {
  const { removeFriend } = useFriendStore();
  const openDM = useNavigationStore((s) => s.openDM);
  const [contextMenu, setContextMenu] = useState<{ friendNumber: number; x: number; y: number } | null>(null);

  if (friends.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-discord-muted">
        <p className="text-sm">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className="p-4" onClick={() => setContextMenu(null)}>
      <h4 className="mb-3 text-xs font-bold uppercase text-discord-muted">
        {label}
      </h4>
      <div className="space-y-px">
        {friends.map((friend) => (
          <div
            key={friend.friend_number}
            className="group flex items-center rounded-lg p-2.5 transition-colors hover:bg-discord-hover"
            onContextMenu={(e) => {
              e.preventDefault();
              setContextMenu({ friendNumber: friend.friend_number, x: e.clientX, y: e.clientY });
            }}
          >
            {/* Avatar with status indicator */}
            <div className="relative mr-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-blurple text-sm font-bold text-white">
                {(friend.name || "?")[0]?.toUpperCase()}
              </div>
              <StatusIndicator status={friend.connection_status} userStatus={friend.user_status} />
            </div>

            {/* Info */}
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium text-white">
                {friend.name || friend.public_key.slice(0, 16) + "..."}
              </p>
              <p className="truncate text-xs text-discord-muted">
                {friend.connection_status !== "none"
                  ? friend.status_message || statusLabel(friend.user_status)
                  : friend.last_seen
                    ? `Last seen ${formatLastSeen(friend.last_seen)}`
                    : "Offline"}
              </p>
            </div>

            {/* Actions (visible on hover) */}
            <div className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  openDM(friend.friend_number);
                }}
                className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-darker text-discord-muted hover:text-white"
                title="Message"
              >
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                </svg>
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setContextMenu({ friendNumber: friend.friend_number, x: e.clientX, y: e.clientY });
                }}
                className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-darker text-discord-muted hover:text-white"
                title="More"
              >
                <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                </svg>
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Context menu */}
      {contextMenu && (
        <div
          className="fixed z-50 w-48 rounded-md bg-discord-darker py-1 shadow-xl"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            onClick={() => {
              openDM(contextMenu.friendNumber);
              setContextMenu(null);
            }}
            className="w-full px-3 py-1.5 text-left text-sm text-discord-text hover:bg-discord-blurple hover:text-white"
          >
            Message
          </button>
          <button
            onClick={() => {
              removeFriend(contextMenu.friendNumber);
              setContextMenu(null);
            }}
            className="w-full px-3 py-1.5 text-left text-sm text-discord-red hover:bg-discord-red hover:text-white"
          >
            Remove Friend
          </button>
        </div>
      )}
    </div>
  );
}

function StatusIndicator({
  status,
  userStatus,
}: {
  status: string;
  userStatus: string;
}) {
  let color = "bg-discord-muted"; // offline
  if (status !== "none") {
    switch (userStatus) {
      case "online":
      case "none":
        color = "bg-discord-green";
        break;
      case "away":
        color = "bg-discord-yellow";
        break;
      case "busy":
        color = "bg-discord-red";
        break;
    }
  }

  return (
    <div
      className={`absolute -bottom-0.5 -right-0.5 h-3.5 w-3.5 rounded-full border-2 border-discord-chat ${color}`}
    />
  );
}

function statusLabel(status: string): string {
  switch (status) {
    case "away":
      return "Away";
    case "busy":
      return "Do Not Disturb";
    default:
      return "Online";
  }
}

function formatLastSeen(isoDate: string): string {
  try {
    const date = new Date(isoDate);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return "just now";
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return date.toLocaleDateString();
  } catch {
    return "unknown";
  }
}

function CreateGroupModal({ onClose }: { onClose: () => void }) {
  const [name, setName] = useState("");
  const [selectedFriends, setSelectedFriends] = useState<number[]>([]);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState("");
  const { friends, loadFriends } = useFriendStore();
  const createDmGroup = useGuildStore((s) => s.createDmGroup);
  const openDmGroup = useNavigationStore((s) => s.openDmGroup);

  useEffect(() => {
    loadFriends();
  }, [loadFriends]);

  const onlineFriends = friends.filter((f) => f.connection_status !== "none");

  const handleCreate = async () => {
    const trimmed = name.trim();
    if (!trimmed || selectedFriends.length === 0) return;

    setIsCreating(true);
    setError("");
    try {
      const dmGroup = await createDmGroup(trimmed, selectedFriends);
      openDmGroup(dmGroup.id);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setIsCreating(false);
    }
  };

  const toggleFriend = (friendNumber: number) => {
    setSelectedFriends((prev) =>
      prev.includes(friendNumber)
        ? prev.filter((n) => n !== friendNumber)
        : [...prev, friendNumber]
    );
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-[440px] rounded-lg bg-discord-sidebar p-6">
        <h2 className="mb-1 text-center text-2xl font-bold text-white">
          Create a Group DM
        </h2>
        <p className="mb-6 text-center text-sm text-discord-muted">
          Start a group conversation with friends.
        </p>

        <div className="mb-4">
          <label className="mb-2 block text-xs font-bold uppercase text-discord-muted">
            Group Name
          </label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Enter group name"
            className="w-full rounded-md bg-discord-input px-3 py-2 text-sm text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
            autoFocus
            maxLength={48}
          />
        </div>

        <div className="mb-4">
          <label className="mb-2 block text-xs font-bold uppercase text-discord-muted">
            Select Friends ({selectedFriends.length} selected)
          </label>
          <div className="max-h-48 overflow-y-auto rounded-md bg-discord-input p-2">
            {onlineFriends.length === 0 ? (
              <p className="py-4 text-center text-sm text-discord-muted">
                No online friends available
              </p>
            ) : (
              onlineFriends.map((friend) => (
                <label
                  key={friend.friend_number}
                  className="flex cursor-pointer items-center gap-3 rounded-md p-2 hover:bg-discord-hover"
                >
                  <input
                    type="checkbox"
                    checked={selectedFriends.includes(friend.friend_number)}
                    onChange={() => toggleFriend(friend.friend_number)}
                    className="h-4 w-4 rounded border-discord-muted bg-discord-input text-discord-blurple focus:ring-discord-blurple"
                  />
                  <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-blurple text-xs font-bold text-white">
                      {friend.name[0]?.toUpperCase() ?? "?"}
                    </div>
                    <span className="text-sm text-white">{friend.name}</span>
                  </div>
                </label>
              ))
            )}
          </div>
        </div>

        {error && (
          <p className="mb-4 text-sm text-discord-red">{error}</p>
        )}

        <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="rounded-md px-4 py-2 text-sm font-medium text-discord-muted hover:text-white"
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={!name.trim() || isCreating || selectedFriends.length === 0}
            className="rounded-md bg-discord-blurple px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:opacity-50"
          >
            {isCreating ? "Creating..." : "Create Group DM"}
          </button>
        </div>
      </div>
    </div>
  );
}
