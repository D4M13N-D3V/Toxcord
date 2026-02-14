import { useState } from "react";
import { useAuthStore } from "../../stores/authStore";
import { useNavigationStore } from "../../stores/navigationStore";
import { GuildCreateModal } from "../guild/GuildCreateModal";

export function MainContent() {
  const { toxId, isConnected, connectionType, displayName } = useAuthStore();
  const setPage = useNavigationStore((s) => s.setPage);
  const [copied, setCopied] = useState(false);
  const [showCreateGuild, setShowCreateGuild] = useState(false);

  const copyToxId = async () => {
    if (toxId) {
      await navigator.clipboard.writeText(toxId);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="flex flex-1 flex-col bg-discord-chat">
      {/* Header */}
      <div className="flex h-12 items-center px-4">
        <h3 className="font-semibold text-white">Home</h3>
      </div>

      {/* Welcome content */}
      <div className="flex flex-1 items-center justify-center">
        <div className="max-w-lg text-center">
          <div className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-discord-blurple">
            <span className="text-3xl font-bold text-white">T</span>
          </div>

          <h2 className="mb-2 text-2xl font-bold text-white">
            Welcome to Toxcord, {displayName}!
          </h2>

          <p className="mb-6 text-discord-muted">
            You're connected to the TOX peer-to-peer network.
            <br />
            All your messages are end-to-end encrypted.
          </p>

          {/* Connection Status */}
          <div className="mb-6 rounded-lg bg-discord-sidebar p-4">
            <div className="mb-3 flex items-center justify-center gap-2">
              <div
                className={`h-3 w-3 rounded-full ${
                  isConnected ? "bg-discord-green" : "bg-discord-yellow animate-pulse"
                }`}
              />
              <span className="text-sm font-medium text-white">
                {isConnected
                  ? `Connected via ${connectionType.toUpperCase()}`
                  : "Connecting to TOX network..."}
              </span>
            </div>

            {toxId && (
              <div>
                <p className="mb-2 text-xs font-bold uppercase text-discord-muted">
                  Your Tox ID
                </p>
                <button
                  onClick={copyToxId}
                  className="group w-full rounded-md bg-discord-darker p-3 text-left transition-colors hover:bg-discord-input"
                >
                  <code className="break-all text-xs text-discord-text">
                    {toxId}
                  </code>
                  <p className="mt-1 text-xs text-discord-muted">
                    {copied
                      ? "Copied!"
                      : "Click to copy — share this with friends to connect"}
                  </p>
                </button>
              </div>
            )}
          </div>

          {/* Quick actions */}
          <div className="grid grid-cols-2 gap-3">
            <button
              onClick={() => setPage("friends")}
              className="rounded-lg bg-discord-sidebar p-4 text-left transition-colors hover:bg-discord-hover"
            >
              <div className="mb-1 text-sm font-medium text-white">
                Add Friend
              </div>
              <div className="text-xs text-discord-muted">Connect with a Tox ID</div>
            </button>
            <button
              onClick={() => setShowCreateGuild(true)}
              className="rounded-lg bg-discord-sidebar p-4 text-left transition-colors hover:bg-discord-hover"
            >
              <div className="mb-1 text-sm font-medium text-white">
                Create Community
              </div>
              <div className="text-xs text-discord-muted">Start a new community</div>
            </button>
          </div>
        </div>
      </div>

      {showCreateGuild && (
        <GuildCreateModal onClose={() => setShowCreateGuild(false)} />
      )}
    </div>
  );
}
