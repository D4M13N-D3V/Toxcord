import { useState, useEffect } from "react";
import { useGuildStore } from "../../stores/guildStore";
import { useFriendStore } from "../../stores/friendStore";
import { useNavigationStore } from "../../stores/navigationStore";

type Mode = "select" | "server" | "dm_group";

export function GuildCreateModal({ onClose }: { onClose: () => void }) {
  const [mode, setMode] = useState<Mode>("select");
  const [name, setName] = useState("");
  const [selectedFriends, setSelectedFriends] = useState<number[]>([]);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState("");
  const createGuild = useGuildStore((s) => s.createGuild);
  const createDmGroup = useGuildStore((s) => s.createDmGroup);
  const friends = useFriendStore((s) => s.friends);
  const loadFriends = useFriendStore((s) => s.loadFriends);
  const openDmGroup = useNavigationStore((s) => s.openDmGroup);

  useEffect(() => {
    loadFriends();
  }, [loadFriends]);

  const onlineFriends = friends.filter((f) => f.connection_status !== "none");

  const handleCreate = async () => {
    const trimmed = name.trim();
    if (!trimmed) return;

    setIsCreating(true);
    setError("");
    try {
      if (mode === "server") {
        await createGuild(trimmed);
      } else if (mode === "dm_group") {
        if (selectedFriends.length === 0) {
          setError("Please select at least one friend");
          setIsCreating(false);
          return;
        }
        const dmGroup = await createDmGroup(trimmed, selectedFriends);
        openDmGroup(dmGroup.id);
      }
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

  // Selection mode
  if (mode === "select") {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
        <div className="w-[440px] rounded-lg bg-discord-sidebar p-6">
          <h2 className="mb-1 text-center text-2xl font-bold text-white">
            Create Something New
          </h2>
          <p className="mb-6 text-center text-sm text-discord-muted">
            What would you like to create?
          </p>

          <div className="space-y-3">
            <button
              onClick={() => setMode("server")}
              className="flex w-full items-center gap-3 rounded-lg bg-discord-channel p-4 text-left transition-colors hover:bg-discord-hover"
            >
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-discord-blurple">
                <svg className="h-6 w-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
                </svg>
              </div>
              <div className="min-w-0 flex-1">
                <p className="font-semibold text-white">Create a Community</p>
                <p className="text-sm text-discord-muted">
                  A persistent server with channels and roles
                </p>
              </div>
              <svg className="h-5 w-5 text-discord-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>

            <button
              onClick={() => setMode("dm_group")}
              className="flex w-full items-center gap-3 rounded-lg bg-discord-channel p-4 text-left transition-colors hover:bg-discord-hover"
            >
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-discord-green">
                <svg className="h-6 w-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
                </svg>
              </div>
              <div className="min-w-0 flex-1">
                <p className="font-semibold text-white">Create a Group DM</p>
                <p className="text-sm text-discord-muted">
                  A group chat with selected friends
                </p>
              </div>
              <svg className="h-5 w-5 text-discord-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>

          <div className="mt-6 flex justify-end">
            <button
              onClick={onClose}
              className="rounded-md px-4 py-2 text-sm font-medium text-discord-muted hover:text-white"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Server or DM Group creation mode
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-[440px] rounded-lg bg-discord-sidebar p-6">
        <button
          onClick={() => setMode("select")}
          className="mb-4 flex items-center gap-1 text-sm text-discord-muted hover:text-white"
        >
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
          Back
        </button>

        <h2 className="mb-1 text-center text-2xl font-bold text-white">
          {mode === "server" ? "Create a Community" : "Create a Group DM"}
        </h2>
        <p className="mb-6 text-center text-sm text-discord-muted">
          {mode === "server"
            ? "Your server is where you and your friends hang out."
            : "Start a group conversation with friends."}
        </p>

        <div className="mb-4">
          <label className="mb-2 block text-xs font-bold uppercase text-discord-muted">
            {mode === "server" ? "Community Name" : "Group Name"}
          </label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && mode === "server" && handleCreate()}
            placeholder={mode === "server" ? "Enter community name" : "Enter group name"}
            className="w-full rounded-md bg-discord-input px-3 py-2 text-sm text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
            autoFocus
            maxLength={48}
          />
        </div>

        {mode === "dm_group" && (
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
        )}

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
            disabled={!name.trim() || isCreating || (mode === "dm_group" && selectedFriends.length === 0)}
            className="rounded-md bg-discord-blurple px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:opacity-50"
          >
            {isCreating ? "Creating..." : "Create"}
          </button>
        </div>
      </div>
    </div>
  );
}
