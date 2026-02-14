import { useEffect, useState } from "react";
import { useAuthStore } from "../../stores/authStore";
import { useFriendStore } from "../../stores/friendStore";
import { useNavigationStore } from "../../stores/navigationStore";
import { useMessageStore } from "../../stores/messageStore";
import { useGuildStore } from "../../stores/guildStore";
import { InviteModal } from "../guild/InviteModal";
import * as api from "../../api/tox";

const EMPTY_CHANNELS: never[] = [];

export function ChannelSidebar() {
  const selectedGuildId = useNavigationStore((s) => s.selectedGuildId);
  const currentPage = useNavigationStore((s) => s.currentPage);

  // Show guild channel sidebar only for regular guilds, not DM groups
  if (currentPage === "guild" && selectedGuildId) {
    return <GuildChannelSidebar guildId={selectedGuildId} />;
  }

  // Show DM sidebar for home, friends, dm, and dm_group pages
  return <DMSidebar />;
}

function GuildChannelSidebar({ guildId }: { guildId: string }) {
  const { displayName, statusMessage, isConnected, connectionType, logout } =
    useAuthStore();
  const guilds = useGuildStore((s) => s.guilds);
  const channels = useGuildStore((s) => s.channels[guildId] ?? EMPTY_CHANNELS);
  const loadChannels = useGuildStore((s) => s.loadChannels);
  const loadMembers = useGuildStore((s) => s.loadMembers);
  const createChannel = useGuildStore((s) => s.createChannel);
  const renameGuild = useGuildStore((s) => s.renameGuild);
  const renameChannel = useGuildStore((s) => s.renameChannel);
  const leaveGuild = useGuildStore((s) => s.leaveGuild);
  const selectedChannelId = useNavigationStore((s) => s.selectedChannelId);
  const openChannel = useNavigationStore((s) => s.openChannel);
  const setPage = useNavigationStore((s) => s.setPage);

  const [showInviteModal, setShowInviteModal] = useState(false);
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [newChannelName, setNewChannelName] = useState("");
  const [showGuildMenu, setShowGuildMenu] = useState(false);
  const [isRenamingGuild, setIsRenamingGuild] = useState(false);
  const [guildNameEdit, setGuildNameEdit] = useState("");
  const [renamingChannelId, setRenamingChannelId] = useState<string | null>(null);
  const [channelNameEdit, setChannelNameEdit] = useState("");

  const [isEditingName, setIsEditingName] = useState(false);
  const [editName, setEditName] = useState(displayName ?? "");
  const [isEditingStatus, setIsEditingStatus] = useState(false);
  const [editStatus, setEditStatus] = useState(statusMessage ?? "");

  const guild = guilds.find((g) => g.id === guildId);

  useEffect(() => {
    loadChannels(guildId).then(() => {
      const navState = useNavigationStore.getState();
      if (!navState.selectedChannelId) {
        const loaded = useGuildStore.getState().channels[guildId];
        if (loaded && loaded.length > 0) {
          openChannel(guildId, loaded[0].id);
        }
      }
    });
    loadMembers(guildId);
  }, [guildId, loadChannels, loadMembers, openChannel]);

  const handleCreateChannel = async () => {
    const trimmed = newChannelName.trim();
    if (!trimmed) return;
    try {
      await createChannel(guildId, trimmed);
      setNewChannelName("");
      setShowCreateChannel(false);
    } catch {
      // ignore
    }
  };

  const handleGuildRename = async () => {
    const trimmed = guildNameEdit.trim();
    if (trimmed && trimmed !== guild?.name) {
      try {
        await renameGuild(guildId, trimmed);
      } catch {
        // ignore
      }
    }
    setIsRenamingGuild(false);
  };

  const handleChannelRename = async (channelId: string) => {
    const trimmed = channelNameEdit.trim();
    if (trimmed && trimmed !== channels.find((c) => c.id === channelId)?.name) {
      try {
        await renameChannel(guildId, channelId, trimmed);
      } catch {
        // ignore
      }
    }
    setRenamingChannelId(null);
  };

  const handleLeaveGuild = async () => {
    setShowGuildMenu(false);
    if (confirm("Are you sure you want to leave this server?")) {
      await leaveGuild(guildId);
      setPage("home");
    }
  };

  const handleDeleteGuild = async () => {
    setShowGuildMenu(false);
    if (confirm("Are you sure you want to delete this server? All data will be lost.")) {
      await leaveGuild(guildId);
      setPage("home");
    }
  };

  const handleNameSave = async () => {
    if (editName.trim() && editName !== displayName) {
      try {
        await api.setDisplayName(editName.trim());
        useAuthStore.setState({ displayName: editName.trim() });
      } catch {
        setEditName(displayName ?? "");
      }
    }
    setIsEditingName(false);
  };

  const handleStatusSave = async () => {
    if (editStatus !== statusMessage) {
      try {
        await api.setStatusMessage(editStatus.trim());
        useAuthStore.setState({ statusMessage: editStatus.trim() });
      } catch {
        setEditStatus(statusMessage ?? "");
      }
    }
    setIsEditingStatus(false);
  };

  return (
    <>
      <div className="flex w-60 flex-col bg-discord-sidebar">
        {/* Guild header with dropdown */}
        <div className="relative">
          <button
            onClick={() => setShowGuildMenu(!showGuildMenu)}
            className="flex h-12 w-full items-center justify-between px-4 transition-colors hover:bg-discord-hover/50"
          >
            {isRenamingGuild ? (
              <input
                value={guildNameEdit}
                onChange={(e) => setGuildNameEdit(e.target.value)}
                onBlur={handleGuildRename}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleGuildRename();
                  if (e.key === "Escape") setIsRenamingGuild(false);
                }}
                onClick={(e) => e.stopPropagation()}
                className="w-full rounded bg-discord-input px-2 py-0.5 text-sm font-semibold text-white outline-none"
                autoFocus
              />
            ) : (
              <h2 className="truncate font-semibold text-white">
                {guild?.name ?? "Community"}
              </h2>
            )}
            <svg
              className={`h-4 w-4 flex-shrink-0 text-discord-muted transition-transform ${showGuildMenu ? "rotate-180" : ""}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          {showGuildMenu && (
            <>
              <div className="fixed inset-0 z-40" onClick={() => setShowGuildMenu(false)} />
              <div className="absolute left-2 right-2 top-12 z-50 rounded-md bg-discord-darker py-1 shadow-lg">
                <button
                  onClick={() => { setShowInviteModal(true); setShowGuildMenu(false); }}
                  className="flex w-full items-center px-3 py-2 text-sm text-discord-muted hover:bg-discord-blurple hover:text-white"
                >
                  <svg className="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M19 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zM4 19.235v-.11a6.375 6.375 0 0112.75 0v.109A12.318 12.318 0 0110.374 21c-2.331 0-4.512-.645-6.374-1.766z" />
                  </svg>
                  Invite People
                </button>
                <button
                  onClick={() => {
                    setGuildNameEdit(guild?.name ?? "");
                    setIsRenamingGuild(true);
                    setShowGuildMenu(false);
                  }}
                  className="flex w-full items-center px-3 py-2 text-sm text-discord-muted hover:bg-discord-blurple hover:text-white"
                >
                  <svg className="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125" />
                  </svg>
                  Rename Server
                </button>
                <div className="my-1 mx-2 border-t border-discord-channel" />
                <button
                  onClick={handleLeaveGuild}
                  className="flex w-full items-center px-3 py-2 text-sm text-discord-red hover:bg-discord-red/20"
                >
                  <svg className="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                  </svg>
                  Leave Server
                </button>
                <button
                  onClick={handleDeleteGuild}
                  className="flex w-full items-center px-3 py-2 text-sm text-discord-red hover:bg-discord-red/20"
                >
                  <svg className="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                  Delete Server
                </button>
              </div>
            </>
          )}
        </div>

        {/* Channel list */}
        <div className="flex-1 overflow-y-auto px-2 py-3">
          <div className="mb-1 flex items-center px-2">
            <span className="flex-1 text-xs font-semibold uppercase text-discord-muted">
              Text Channels
            </span>
            <button
              onClick={() => setShowCreateChannel(true)}
              className="rounded text-discord-muted hover:text-white"
              title="Create Channel"
            >
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
              </svg>
            </button>
          </div>

          {showCreateChannel && (
            <div className="mb-2 flex gap-1 px-1">
              <input
                value={newChannelName}
                onChange={(e) => setNewChannelName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleCreateChannel();
                  if (e.key === "Escape") setShowCreateChannel(false);
                }}
                placeholder="channel-name"
                className="flex-1 rounded bg-discord-input px-2 py-1 text-xs text-white outline-none"
                autoFocus
              />
            </div>
          )}

          <div className="space-y-px">
            {channels.map((channel) => {
              const isSelected = selectedChannelId === channel.id;
              const isRenaming = renamingChannelId === channel.id;

              if (isRenaming) {
                return (
                  <div key={channel.id} className="flex items-center gap-1.5 rounded-md bg-discord-active px-2 py-1.5">
                    <span className="text-lg leading-none text-discord-muted">#</span>
                    <input
                      value={channelNameEdit}
                      onChange={(e) => setChannelNameEdit(e.target.value)}
                      onBlur={() => handleChannelRename(channel.id)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") handleChannelRename(channel.id);
                        if (e.key === "Escape") setRenamingChannelId(null);
                      }}
                      className="min-w-0 flex-1 rounded bg-discord-input px-1 py-0 text-sm text-white outline-none"
                      autoFocus
                    />
                  </div>
                );
              }

              return (
                <button
                  key={channel.id}
                  onClick={() => openChannel(guildId, channel.id)}
                  onDoubleClick={() => {
                    setRenamingChannelId(channel.id);
                    setChannelNameEdit(channel.name);
                  }}
                  className={`group/ch flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left transition-colors ${
                    isSelected
                      ? "bg-discord-active text-white"
                      : "text-discord-muted hover:bg-discord-hover hover:text-white"
                  }`}
                >
                  <span className="text-lg leading-none text-discord-muted">#</span>
                  <span className="min-w-0 flex-1 truncate text-sm">
                    {channel.name}
                  </span>
                  <svg
                    onClick={(e) => {
                      e.stopPropagation();
                      setRenamingChannelId(channel.id);
                      setChannelNameEdit(channel.name);
                    }}
                    className="hidden h-3.5 w-3.5 flex-shrink-0 text-discord-muted hover:text-white group-hover/ch:block"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    strokeWidth={2}
                  >
                    <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125" />
                  </svg>
                </button>
              );
            })}
          </div>
        </div>

        {/* User panel */}
        <UserPanel
          displayName={displayName}
          statusMessage={statusMessage}
          isConnected={isConnected}
          connectionType={connectionType}
          isEditingName={isEditingName}
          setIsEditingName={setIsEditingName}
          editName={editName}
          setEditName={setEditName}
          handleNameSave={handleNameSave}
          isEditingStatus={isEditingStatus}
          setIsEditingStatus={setIsEditingStatus}
          editStatus={editStatus}
          setEditStatus={setEditStatus}
          handleStatusSave={handleStatusSave}
          logout={logout}
        />
      </div>

      {showInviteModal && (
        <InviteModal guildId={guildId} onClose={() => setShowInviteModal(false)} />
      )}
    </>
  );
}

function DMSidebar() {
  const { displayName, statusMessage, isConnected, connectionType, logout } =
    useAuthStore();
  const { friends, friendRequests, loadFriends, loadFriendRequests } =
    useFriendStore();
  const { currentPage, selectedFriendNumber, selectedDmGroupId, setPage, openDM, openDmGroup } =
    useNavigationStore();
  const { dmGroups, loadDmGroups } = useGuildStore();
  const unreadCounts = useMessageStore((s) => s.unreadCounts);

  const [isEditingName, setIsEditingName] = useState(false);
  const [editName, setEditName] = useState(displayName ?? "");
  const [isEditingStatus, setIsEditingStatus] = useState(false);
  const [editStatus, setEditStatus] = useState(statusMessage ?? "");
  const [showGroupModal, setShowGroupModal] = useState(false);

  const pendingCount = friendRequests.length;

  useEffect(() => {
    loadFriends();
    loadFriendRequests();
    loadDmGroups();
  }, [loadFriends, loadFriendRequests, loadDmGroups]);

  const handleNameSave = async () => {
    if (editName.trim() && editName !== displayName) {
      try {
        await api.setDisplayName(editName.trim());
        useAuthStore.setState({ displayName: editName.trim() });
      } catch {
        setEditName(displayName ?? "");
      }
    }
    setIsEditingName(false);
  };

  const handleStatusSave = async () => {
    if (editStatus !== statusMessage) {
      try {
        await api.setStatusMessage(editStatus.trim());
        useAuthStore.setState({ statusMessage: editStatus.trim() });
      } catch {
        setEditStatus(statusMessage ?? "");
      }
    }
    setIsEditingStatus(false);
  };

  return (
    <div className="flex w-60 flex-col bg-discord-sidebar">
      {/* Header */}
      <div className="flex h-12 items-center px-4">
        <h2 className="truncate font-semibold text-white">Toxcord</h2>
      </div>

      {/* Navigation */}
      <div className="flex-1 overflow-y-auto px-2 py-3">
        {/* Friends button */}
        <button
          onClick={() => setPage("friends")}
          className={`mb-1 flex w-full items-center gap-3 rounded-md px-2.5 py-2 text-left transition-colors ${
            currentPage === "friends"
              ? "bg-discord-active text-white"
              : "text-discord-muted hover:bg-discord-hover hover:text-white"
          }`}
        >
          <svg
            className="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-1.053M18 8.625a4.5 4.5 0 11-9 0 4.5 4.5 0 019 0z"
            />
          </svg>
          <span className="text-sm font-medium">Friends</span>
          {pendingCount > 0 && (
            <span className="ml-auto flex h-4 min-w-4 items-center justify-center rounded-full bg-discord-red px-1 text-xs font-bold text-white">
              {pendingCount}
            </span>
          )}
        </button>

        {/* DM section header */}
        <div className="mb-1 mt-4 flex items-center px-2">
          <span className="flex-1 text-xs font-semibold uppercase text-discord-muted">
            Direct Messages
          </span>
          <button
            onClick={() => setShowGroupModal(true)}
            className="rounded text-discord-muted hover:text-white"
            title="Create Group DM"
          >
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>

        {/* Friends as DM shortcuts */}
        {friends.length > 0 ? (
          <div className="space-y-px">
            {[...friends]
              .sort((a, b) => {
                const aOnline = a.connection_status !== "none" ? 1 : 0;
                const bOnline = b.connection_status !== "none" ? 1 : 0;
                if (aOnline !== bOnline) return bOnline - aOnline;
                const aUnread = unreadCounts[a.friend_number] ?? 0;
                const bUnread = unreadCounts[b.friend_number] ?? 0;
                if (aUnread !== bUnread) return bUnread - aUnread;
                return 0;
              })
              .map((friend) => {
                const isSelected =
                  currentPage === "dm" &&
                  selectedFriendNumber === friend.friend_number;
                const unread = unreadCounts[friend.friend_number] ?? 0;
                const isOnline = friend.connection_status !== "none";

                return (
                  <button
                    key={friend.friend_number}
                    onClick={() => openDM(friend.friend_number)}
                    className={`flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left transition-colors ${
                      isSelected
                        ? "bg-discord-active text-white"
                        : "text-discord-muted hover:bg-discord-hover hover:text-white"
                    }`}
                  >
                    <div className="relative">
                      <div className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-blurple text-xs font-bold text-white">
                        {(friend.name || "?")[0]?.toUpperCase()}
                      </div>
                      <div
                        className={`absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-discord-sidebar ${
                          isOnline ? "bg-discord-green" : "bg-discord-muted"
                        }`}
                      />
                    </div>
                    <span className="min-w-0 flex-1 truncate text-sm">
                      {friend.name || friend.public_key.slice(0, 8) + "..."}
                    </span>
                    {unread > 0 && (
                      <span className="flex h-4 min-w-4 items-center justify-center rounded-full bg-discord-red px-1 text-xs font-bold text-white">
                        {unread}
                      </span>
                    )}
                  </button>
                );
              })}
          </div>
        ) : (
          <p className="px-2 text-xs text-discord-muted">No friends online</p>
        )}

        {/* Group DMs section */}
        {dmGroups.length > 0 && (
          <>
            <div className="mb-1 mt-4 flex items-center px-2">
              <span className="flex-1 text-xs font-semibold uppercase text-discord-muted">
                Group Messages
              </span>
            </div>
            <div className="space-y-px">
              {dmGroups.map((group) => {
                const isSelected = selectedDmGroupId === group.id;
                return (
                  <button
                    key={group.id}
                    onClick={() => openDmGroup(group.id)}
                    className={`flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left transition-colors ${
                      isSelected
                        ? "bg-discord-active text-white"
                        : "text-discord-muted hover:bg-discord-hover hover:text-white"
                    }`}
                  >
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-green text-xs font-bold text-white">
                      <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
                      </svg>
                    </div>
                    <span className="min-w-0 flex-1 truncate text-sm">
                      {group.name}
                    </span>
                  </button>
                );
              })}
            </div>
          </>
        )}
      </div>

      {/* User panel */}
      <UserPanel
        displayName={displayName}
        statusMessage={statusMessage}
        isConnected={isConnected}
        connectionType={connectionType}
        isEditingName={isEditingName}
        setIsEditingName={setIsEditingName}
        editName={editName}
        setEditName={setEditName}
        handleNameSave={handleNameSave}
        isEditingStatus={isEditingStatus}
        setIsEditingStatus={setIsEditingStatus}
        editStatus={editStatus}
        setEditStatus={setEditStatus}
        handleStatusSave={handleStatusSave}
        logout={logout}
      />

      {showGroupModal && (
        <CreateGroupDMModal onClose={() => setShowGroupModal(false)} />
      )}
    </div>
  );
}

function UserPanel({
  displayName,
  statusMessage,
  isConnected,
  connectionType,
  isEditingName,
  setIsEditingName,
  editName,
  setEditName,
  handleNameSave,
  isEditingStatus,
  setIsEditingStatus,
  editStatus,
  setEditStatus,
  handleStatusSave,
  logout,
}: {
  displayName: string | null;
  statusMessage: string | null;
  isConnected: boolean;
  connectionType: string;
  isEditingName: boolean;
  setIsEditingName: (v: boolean) => void;
  editName: string;
  setEditName: (v: string) => void;
  handleNameSave: () => void;
  isEditingStatus: boolean;
  setIsEditingStatus: (v: boolean) => void;
  editStatus: string;
  setEditStatus: (v: string) => void;
  handleStatusSave: () => void;
  logout: () => void;
}) {
  return (
    <div className="bg-discord-darker/50 px-2 py-1.5">
      <div className="flex items-center">
        <div className="relative mr-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-blurple text-sm font-bold text-white">
            {displayName?.[0]?.toUpperCase() ?? "?"}
          </div>
          <div
            className={`absolute -bottom-0.5 -right-0.5 h-3.5 w-3.5 rounded-full border-2 border-discord-darker ${
              isConnected ? "bg-discord-green" : "bg-discord-muted"
            }`}
          />
        </div>

        <div className="min-w-0 flex-1">
          {isEditingName ? (
            <input
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              onBlur={handleNameSave}
              onKeyDown={(e) => e.key === "Enter" && handleNameSave()}
              className="w-full rounded bg-discord-input px-1 text-sm text-white outline-none"
              autoFocus
            />
          ) : (
            <div
              className="cursor-pointer truncate text-sm font-medium text-white hover:underline"
              onClick={() => {
                setEditName(displayName ?? "");
                setIsEditingName(true);
              }}
              title="Click to edit display name"
            >
              {displayName}
            </div>
          )}
          {isEditingStatus ? (
            <input
              value={editStatus}
              onChange={(e) => setEditStatus(e.target.value)}
              onBlur={handleStatusSave}
              onKeyDown={(e) => e.key === "Enter" && handleStatusSave()}
              className="w-full rounded bg-discord-input px-1 text-xs text-discord-muted outline-none"
              autoFocus
              placeholder="Set a status"
            />
          ) : (
            <div
              className="cursor-pointer truncate text-xs text-discord-muted hover:underline"
              onClick={() => {
                setEditStatus(statusMessage ?? "");
                setIsEditingStatus(true);
              }}
              title="Click to edit status"
            >
              {isConnected
                ? statusMessage || `Connected (${connectionType.toUpperCase()})`
                : "Connecting..."}
            </div>
          )}
        </div>

        <button
          onClick={logout}
          className="ml-1 rounded p-1 text-discord-muted hover:bg-discord-hover hover:text-white"
          title="Logout"
        >
          <svg
            className="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15m3 0l3-3m0 0l-3-3m3 3H9"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}

function CreateGroupDMModal({ onClose }: { onClose: () => void }) {
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
