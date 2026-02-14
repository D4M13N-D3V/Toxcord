import { useEffect, useState } from "react";
import { useAuthStore } from "../stores/authStore";

export function LoginPage() {
  const {
    profiles,
    isLoading,
    error,
    loadProfiles,
    createProfile,
    loadProfile,
    deleteProfile,
    clearError,
  } = useAuthStore();

  const [mode, setMode] = useState<"select" | "create" | "login">("select");
  const [profileName, setProfileName] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [selectedProfile, setSelectedProfile] = useState<string | null>(null);
  const [profileToDelete, setProfileToDelete] = useState<string | null>(null);

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!profileName.trim() || !displayName.trim()) return;
    await createProfile(profileName.trim(), password, displayName.trim());
  };

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedProfile) return;
    await loadProfile(selectedProfile, password);
  };

  return (
    <div className="flex h-screen items-center justify-center bg-discord-darker">
      <div className="w-full max-w-md rounded-lg bg-discord-channel p-8 shadow-xl">
        {/* Logo */}
        <div className="mb-8 text-center">
          <h1 className="text-3xl font-bold text-white">Toxcord</h1>
          <p className="mt-2 text-sm text-discord-muted">
            Decentralized. Encrypted. Yours.
          </p>
        </div>

        {/* Error display */}
        {error && (
          <div className="mb-4 rounded bg-discord-red/20 p-3 text-sm text-discord-red">
            {error}
            <button
              onClick={clearError}
              className="ml-2 text-xs underline hover:no-underline"
            >
              dismiss
            </button>
          </div>
        )}

        {/* Profile selector view */}
        {mode === "select" && (
          <div>
            {profiles.length > 0 && (
              <>
                <h2 className="mb-4 text-lg font-semibold text-white">
                  Select Profile
                </h2>
                <div className="mb-4 space-y-2">
                  {profiles.map((name) => (
                    <div key={name} className="flex items-center gap-2">
                      <button
                        onClick={() => {
                          setSelectedProfile(name);
                          setMode("login");
                        }}
                        className="flex flex-1 items-center rounded-md bg-discord-input p-3 text-left text-white transition-colors hover:bg-discord-hover"
                      >
                        <div className="mr-3 flex h-10 w-10 items-center justify-center rounded-full bg-discord-blurple text-lg font-bold">
                          {name[0]?.toUpperCase()}
                        </div>
                        <span className="font-medium">{name}</span>
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setProfileToDelete(name);
                        }}
                        className="rounded-md bg-discord-input p-3 text-discord-muted transition-colors hover:bg-discord-red hover:text-white"
                        title="Delete profile"
                      >
                        <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                          <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  ))}
                </div>
                <div className="my-4 flex items-center">
                  <div className="flex-1 border-t border-discord-input" />
                  <span className="px-3 text-xs text-discord-muted">or</span>
                  <div className="flex-1 border-t border-discord-input" />
                </div>
              </>
            )}
            <button
              onClick={() => setMode("create")}
              className="w-full rounded-md bg-discord-blurple p-3 font-medium text-white transition-colors hover:bg-discord-blurple/80"
            >
              Create New Profile
            </button>
          </div>
        )}

        {/* Create profile form */}
        {mode === "create" && (
          <form onSubmit={handleCreate}>
            <h2 className="mb-4 text-lg font-semibold text-white">
              Create Profile
            </h2>

            <div className="mb-4">
              <label className="mb-1 block text-xs font-bold uppercase text-discord-muted">
                Profile Name
              </label>
              <input
                type="text"
                value={profileName}
                onChange={(e) => setProfileName(e.target.value)}
                placeholder="my-profile"
                className="w-full rounded-md bg-discord-darker p-2.5 text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
                autoFocus
              />
              <p className="mt-1 text-xs text-discord-muted">
                Used as the filename for your encrypted profile
              </p>
            </div>

            <div className="mb-4">
              <label className="mb-1 block text-xs font-bold uppercase text-discord-muted">
                Display Name
              </label>
              <input
                type="text"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                placeholder="Your Name"
                className="w-full rounded-md bg-discord-darker p-2.5 text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
              />
            </div>

            <div className="mb-6">
              <label className="mb-1 block text-xs font-bold uppercase text-discord-muted">
                Password (optional)
              </label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Encrypt your profile"
                className="w-full rounded-md bg-discord-darker p-2.5 text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
              />
              <p className="mt-1 text-xs text-discord-muted">
                Encrypts your profile data locally
              </p>
            </div>

            <button
              type="submit"
              disabled={isLoading || !profileName.trim() || !displayName.trim()}
              className="w-full rounded-md bg-discord-blurple p-3 font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {isLoading ? "Creating..." : "Create Profile"}
            </button>

            <button
              type="button"
              onClick={() => setMode("select")}
              className="mt-3 w-full text-sm text-discord-muted hover:text-white"
            >
              Back
            </button>
          </form>
        )}

        {/* Login form */}
        {mode === "login" && selectedProfile && (
          <form onSubmit={handleLogin}>
            <h2 className="mb-4 text-lg font-semibold text-white">
              Welcome back
            </h2>
            <div className="mb-4 flex items-center rounded-md bg-discord-input p-3">
              <div className="mr-3 flex h-10 w-10 items-center justify-center rounded-full bg-discord-blurple text-lg font-bold text-white">
                {selectedProfile[0]?.toUpperCase()}
              </div>
              <span className="font-medium text-white">{selectedProfile}</span>
            </div>

            <div className="mb-6">
              <label className="mb-1 block text-xs font-bold uppercase text-discord-muted">
                Password
              </label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter your password"
                className="w-full rounded-md bg-discord-darker p-2.5 text-white placeholder-discord-muted outline-none focus:ring-2 focus:ring-discord-blurple"
                autoFocus
              />
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full rounded-md bg-discord-blurple p-3 font-medium text-white transition-colors hover:bg-discord-blurple/80 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {isLoading ? "Loading..." : "Login"}
            </button>

            <button
              type="button"
              onClick={() => {
                setMode("select");
                setPassword("");
                setSelectedProfile(null);
              }}
              className="mt-3 w-full text-sm text-discord-muted hover:text-white"
            >
              Back
            </button>
          </form>
        )}

        {/* Footer */}
        <div className="mt-6 text-center">
          <p className="text-xs text-discord-muted">
            Powered by the TOX P2P protocol
          </p>
        </div>
      </div>

      {/* Delete confirmation modal */}
      {profileToDelete && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
          <div className="w-[400px] rounded-lg bg-discord-sidebar p-6">
            <h2 className="mb-2 text-xl font-bold text-white">Delete Profile</h2>
            <p className="mb-4 text-sm text-discord-muted">
              Are you sure you want to delete <span className="font-semibold text-white">{profileToDelete}</span>?
              This will permanently delete your Tox identity and all message history. This action cannot be undone.
            </p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setProfileToDelete(null)}
                className="rounded-md px-4 py-2 text-sm font-medium text-discord-muted hover:text-white"
              >
                Cancel
              </button>
              <button
                onClick={async () => {
                  await deleteProfile(profileToDelete);
                  setProfileToDelete(null);
                }}
                disabled={isLoading}
                className="rounded-md bg-discord-red px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-discord-red/80 disabled:opacity-50"
              >
                {isLoading ? "Deleting..." : "Delete"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
