import { useState } from "react";
import { useFriendStore } from "../../stores/friendStore";
import * as api from "../../api/tox";

export function InviteModal({
  guildId,
  onClose,
}: {
  guildId: string;
  onClose: () => void;
}) {
  const friends = useFriendStore((s) => s.friends);
  const [inviting, setInviting] = useState<number | null>(null);
  const [invited, setInvited] = useState<Set<number>>(new Set());
  const [error, setError] = useState("");

  const onlineFriends = friends.filter((f) => f.connection_status !== "none");

  const handleInvite = async (friendNumber: number) => {
    setInviting(friendNumber);
    setError("");
    try {
      await api.inviteToGuild(guildId, friendNumber);
      setInvited((prev) => new Set(prev).add(friendNumber));
    } catch (e) {
      setError(String(e));
    } finally {
      setInviting(null);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-[440px] rounded-lg bg-discord-sidebar p-6">
        <h2 className="mb-1 text-xl font-bold text-white">
          Invite Friends
        </h2>
        <p className="mb-4 text-sm text-discord-muted">
          Invite online friends to your server.
        </p>

        {error && (
          <p className="mb-3 text-sm text-discord-red">{error}</p>
        )}

        <div className="max-h-64 space-y-1 overflow-y-auto">
          {onlineFriends.length === 0 ? (
            <p className="py-4 text-center text-sm text-discord-muted">
              No friends online
            </p>
          ) : (
            onlineFriends.map((friend) => {
              const wasInvited = invited.has(friend.friend_number);
              return (
                <div
                  key={friend.friend_number}
                  className="flex items-center gap-3 rounded-md px-3 py-2 hover:bg-discord-hover"
                >
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-blurple text-xs font-bold text-white">
                    {(friend.name || "?")[0]?.toUpperCase()}
                  </div>
                  <span className="min-w-0 flex-1 truncate text-sm text-white">
                    {friend.name || friend.public_key.slice(0, 8) + "..."}
                  </span>
                  <button
                    onClick={() => handleInvite(friend.friend_number)}
                    disabled={wasInvited || inviting === friend.friend_number}
                    className={`rounded-md border px-3 py-1 text-xs font-medium transition-colors ${
                      wasInvited
                        ? "border-discord-green text-discord-green"
                        : "border-discord-green text-discord-green hover:bg-discord-green hover:text-white"
                    } disabled:opacity-50`}
                  >
                    {wasInvited
                      ? "Invited"
                      : inviting === friend.friend_number
                        ? "..."
                        : "Invite"}
                  </button>
                </div>
              );
            })
          )}
        </div>

        <div className="mt-4 flex justify-end">
          <button
            onClick={onClose}
            className="rounded-md px-4 py-2 text-sm font-medium text-discord-muted hover:text-white"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
}
