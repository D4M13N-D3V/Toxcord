import { useEffect, useState } from "react";
import { useAuthStore } from "../../stores/authStore";
import { useNavigationStore } from "../../stores/navigationStore";
import { useGuildStore } from "../../stores/guildStore";
import { useFriendStore } from "../../stores/friendStore";
import { GuildCreateModal } from "../guild/GuildCreateModal";

export function ServerSidebar() {
  const isConnected = useAuthStore((s) => s.isConnected);
  const { currentPage, setPage, selectedGuildId } = useNavigationStore();
  const openGuild = useNavigationStore((s) => s.openGuild);
  const { guilds, loadGuilds, pendingInvites, acceptInvite, dismissInvite } =
    useGuildStore();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showInvites, setShowInvites] = useState(false);

  useEffect(() => {
    loadGuilds();
  }, [loadGuilds]);

  const isHome = currentPage === "home" || currentPage === "friends" || currentPage === "dm" || currentPage === "dm_group";

  return (
    <>
      <div className="flex w-[72px] flex-col items-center bg-discord-darker py-3">
        {/* Home button */}
        <div className="group relative mb-2">
          {/* Active pill */}
          {isHome && (
            <div className="absolute -left-3 top-1/2 h-10 w-1 -translate-y-1/2 rounded-r-full bg-white" />
          )}
          <button
            onClick={() => setPage("home")}
            className={`flex h-12 w-12 items-center justify-center rounded-2xl transition-all duration-200 hover:rounded-xl hover:bg-discord-blurple ${
              isHome ? "rounded-xl bg-discord-blurple" : "bg-discord-channel"
            }`}
          >
            <svg
              className={`h-6 w-6 ${
                isHome
                  ? "text-white"
                  : "text-discord-muted group-hover:text-white"
              }`}
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M12 3L2 12h3v8h6v-6h2v6h6v-8h3L12 3z" />
            </svg>
          </button>
        </div>

        {/* Divider */}
        <div className="mx-auto mb-2 h-0.5 w-8 rounded-full bg-discord-channel" />

        {/* Pending invite notifications */}
        {pendingInvites.length > 0 && (
          <div className="group relative mb-2">
            <button
              onClick={() => setShowInvites(true)}
              className="relative flex h-12 w-12 items-center justify-center rounded-2xl bg-discord-channel text-discord-green transition-all duration-200 hover:rounded-xl hover:bg-discord-green hover:text-white"
              title={`${pendingInvites.length} pending invite(s)`}
            >
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
              </svg>
              <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-full bg-discord-red px-1 text-xs font-bold text-white">
                {pendingInvites.length}
              </span>
            </button>
          </div>
        )}

        {/* Guild list */}
        <div className="flex-1 space-y-2 overflow-y-auto">
          {guilds.map((guild) => {
            const isActive = selectedGuildId === guild.id;
            // Generate a color from the guild name
            const colors = [
              "bg-discord-blurple",
              "bg-discord-green",
              "bg-discord-red",
              "bg-yellow-600",
              "bg-purple-600",
              "bg-pink-600",
              "bg-teal-600",
            ];
            const colorIndex =
              guild.name.split("").reduce((acc, c) => acc + c.charCodeAt(0), 0) %
              colors.length;
            const bgColor = colors[colorIndex];

            return (
              <div key={guild.id} className="group relative">
                {/* Active pill */}
                {isActive && (
                  <div className="absolute -left-3 top-1/2 h-10 w-1 -translate-y-1/2 rounded-r-full bg-white" />
                )}
                {/* Hover pill */}
                {!isActive && (
                  <div className="absolute -left-3 top-1/2 h-5 w-1 -translate-y-1/2 scale-0 rounded-r-full bg-white transition-transform group-hover:scale-100" />
                )}
                <button
                  onClick={() => openGuild(guild.id)}
                  className={`flex h-12 w-12 items-center justify-center transition-all duration-200 ${
                    isActive
                      ? `rounded-xl ${bgColor}`
                      : `rounded-2xl bg-discord-channel hover:rounded-xl hover:${bgColor}`
                  }`}
                  title={guild.name}
                >
                  <span
                    className={`text-sm font-bold ${
                      isActive
                        ? "text-white"
                        : "text-discord-muted group-hover:text-white"
                    }`}
                  >
                    {guild.name[0]?.toUpperCase() ?? "?"}
                  </span>
                </button>
              </div>
            );
          })}
        </div>

        {/* Add server button */}
        <div className="mt-2">
          <button
            onClick={() => setShowCreateModal(true)}
            className="flex h-12 w-12 items-center justify-center rounded-2xl bg-discord-channel text-discord-green transition-all duration-200 hover:rounded-xl hover:bg-discord-green hover:text-white"
          >
            <svg
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M12 4v16m8-8H4"
              />
            </svg>
          </button>
        </div>

        {/* Settings button */}
        <div className="mt-2">
          <button
            onClick={() => setPage("settings")}
            className={`flex h-12 w-12 items-center justify-center rounded-2xl transition-all duration-200 hover:rounded-xl ${
              currentPage === "settings"
                ? "rounded-xl bg-discord-channel text-white"
                : "text-discord-muted hover:bg-discord-channel hover:text-white"
            }`}
            title="User Settings"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>

        {/* Connection indicator */}
        <div className="mt-2">
          <div
            className={`h-3 w-3 rounded-full ${
              isConnected ? "bg-discord-green" : "bg-discord-red"
            }`}
            title={
              isConnected ? "Connected to TOX network" : "Connecting..."
            }
          />
        </div>
      </div>

      {showCreateModal && (
        <GuildCreateModal onClose={() => setShowCreateModal(false)} />
      )}

      {showInvites && (
        <PendingInvitesModal
          invites={pendingInvites}
          onAccept={async (invite) => {
            try {
              await acceptInvite(invite);
            } catch {
              // error is logged in the store
            }
          }}
          onDismiss={(friendNumber) => dismissInvite(friendNumber)}
          onClose={() => setShowInvites(false)}
        />
      )}
    </>
  );
}

function PendingInvitesModal({
  invites,
  onAccept,
  onDismiss,
  onClose,
}: {
  invites: { friendNumber: number; inviteData: number[]; groupName: string }[];
  onAccept: (invite: { friendNumber: number; inviteData: number[]; groupName: string }) => void;
  onDismiss: (friendNumber: number) => void;
  onClose: () => void;
}) {
  const friends = useFriendStore((s) => s.friends);

  const getFriendName = (friendNumber: number) => {
    const friend = friends.find((f) => f.friend_number === friendNumber);
    return friend?.name || `Friend #${friendNumber}`;
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-[440px] rounded-lg bg-discord-sidebar p-6">
        <h2 className="mb-1 text-xl font-bold text-white">Group Invites</h2>
        <p className="mb-4 text-sm text-discord-muted">
          You have pending group invitations.
        </p>

        <div className="max-h-64 space-y-2 overflow-y-auto">
          {invites.map((invite) => {
            const isDmGroup = invite.groupName.startsWith("[DM]");
            const displayName = isDmGroup
              ? invite.groupName.slice(4) || "DM Group"
              : invite.groupName || "Unknown Group";

            return (
            <div
              key={invite.friendNumber}
              className="flex items-center gap-3 rounded-md bg-discord-chat p-3"
            >
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium text-white">
                  {displayName}
                </p>
                <p className="text-xs text-discord-muted">
                  {isDmGroup ? "DM Group" : "Server"} from {getFriendName(invite.friendNumber)}
                </p>
              </div>
              <button
                onClick={() => onAccept(invite)}
                className="rounded-md bg-discord-green px-3 py-1 text-xs font-medium text-white hover:bg-discord-green/80"
              >
                Join
              </button>
              <button
                onClick={() => onDismiss(invite.friendNumber)}
                className="rounded-md bg-discord-channel px-3 py-1 text-xs font-medium text-discord-muted hover:bg-discord-hover hover:text-white"
              >
                Dismiss
              </button>
            </div>
          );
          })}
        </div>

        <div className="mt-4 flex justify-end">
          <button
            onClick={onClose}
            className="rounded-md px-4 py-2 text-sm font-medium text-discord-muted hover:text-white"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
