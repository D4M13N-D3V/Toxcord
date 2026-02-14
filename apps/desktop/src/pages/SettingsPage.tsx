import { useEffect, useState } from "react";
import { useAuthStore } from "../stores/authStore";
import { useNavigationStore } from "../stores/navigationStore";
import * as api from "../api/tox";

export function SettingsPage() {
  const { displayName, statusMessage } = useAuthStore();
  const setPage = useNavigationStore((s) => s.setPage);

  const [name, setName] = useState(displayName ?? "");
  const [status, setStatus] = useState(statusMessage ?? "");
  const [toxId, setToxId] = useState("");
  const [copied, setCopied] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    api.getToxId().then(setToxId).catch(() => {});
  }, []);

  useEffect(() => {
    setName(displayName ?? "");
  }, [displayName]);

  useEffect(() => {
    setStatus(statusMessage ?? "");
  }, [statusMessage]);

  const handleSaveName = async () => {
    const trimmed = name.trim();
    if (!trimmed || trimmed === displayName) return;
    setSaving(true);
    try {
      await api.setDisplayName(trimmed);
      useAuthStore.setState({ displayName: trimmed });
    } catch {
      setName(displayName ?? "");
    }
    setSaving(false);
  };

  const handleSaveStatus = async () => {
    const trimmed = status.trim();
    if (trimmed === statusMessage) return;
    setSaving(true);
    try {
      await api.setStatusMessage(trimmed);
      useAuthStore.setState({ statusMessage: trimmed });
    } catch {
      setStatus(statusMessage ?? "");
    }
    setSaving(false);
  };

  const handleCopyToxId = () => {
    navigator.clipboard.writeText(toxId);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex flex-1 flex-col bg-discord-chat">
      {/* Header */}
      <div className="flex h-12 items-center justify-between border-b border-discord-darker px-6 shadow-sm">
        <h2 className="font-semibold text-white">User Settings</h2>
        <button
          onClick={() => setPage("home")}
          className="rounded-full p-1.5 text-discord-muted transition-colors hover:bg-discord-hover hover:text-white"
          title="Close settings"
        >
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        <div className="mx-auto max-w-2xl px-6 py-8">
          {/* Profile Section */}
          <section className="mb-10">
            <h3 className="mb-4 text-xs font-bold uppercase text-discord-muted">
              My Account
            </h3>
            <div className="rounded-lg bg-discord-sidebar p-4">
              <div className="flex items-center gap-4 mb-6">
                <div className="flex h-20 w-20 items-center justify-center rounded-full bg-discord-blurple text-2xl font-bold text-white">
                  {(displayName ?? "?")[0]?.toUpperCase()}
                </div>
                <div>
                  <p className="text-lg font-semibold text-white">{displayName}</p>
                  <p className="text-sm text-discord-muted">{statusMessage || "No status set"}</p>
                </div>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="mb-2 block text-xs font-bold uppercase text-discord-muted">
                    Display Name
                  </label>
                  <div className="flex gap-2">
                    <input
                      value={name}
                      onChange={(e) => setName(e.target.value)}
                      onKeyDown={(e) => e.key === "Enter" && handleSaveName()}
                      className="flex-1 rounded-md bg-discord-input px-3 py-2 text-sm text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
                      placeholder="Enter display name"
                      maxLength={32}
                    />
                    <button
                      onClick={handleSaveName}
                      disabled={saving || name.trim() === displayName}
                      className="rounded-md bg-discord-blurple px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:opacity-50"
                    >
                      Save
                    </button>
                  </div>
                </div>

                <div>
                  <label className="mb-2 block text-xs font-bold uppercase text-discord-muted">
                    Status Message
                  </label>
                  <div className="flex gap-2">
                    <input
                      value={status}
                      onChange={(e) => setStatus(e.target.value)}
                      onKeyDown={(e) => e.key === "Enter" && handleSaveStatus()}
                      className="flex-1 rounded-md bg-discord-input px-3 py-2 text-sm text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
                      placeholder="Set a status message"
                      maxLength={128}
                    />
                    <button
                      onClick={handleSaveStatus}
                      disabled={saving || status.trim() === statusMessage}
                      className="rounded-md bg-discord-blurple px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:opacity-50"
                    >
                      Save
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </section>

          {/* Tox ID Section */}
          <section className="mb-10">
            <h3 className="mb-4 text-xs font-bold uppercase text-discord-muted">
              Tox ID
            </h3>
            <div className="rounded-lg bg-discord-sidebar p-4">
              <p className="mb-3 text-sm text-discord-muted">
                Share your Tox ID with friends so they can add you.
              </p>
              <div className="flex gap-2">
                <code className="flex-1 overflow-hidden rounded-md bg-discord-input px-3 py-2 font-mono text-xs text-discord-text break-all select-all">
                  {toxId || "Loading..."}
                </code>
                <button
                  onClick={handleCopyToxId}
                  disabled={!toxId}
                  className="flex-shrink-0 rounded-md bg-discord-blurple px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:opacity-50"
                >
                  {copied ? "Copied!" : "Copy"}
                </button>
              </div>
            </div>
          </section>

          {/* Appearance Section (placeholder) */}
          <section className="mb-10">
            <h3 className="mb-4 text-xs font-bold uppercase text-discord-muted">
              Appearance
            </h3>
            <div className="rounded-lg bg-discord-sidebar p-4">
              <p className="text-sm text-discord-muted">
                Theme customization coming soon.
              </p>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
