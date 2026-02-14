import { useGuildStore } from "../../stores/guildStore";
import { useNavigationStore } from "../../stores/navigationStore";

const EMPTY_MEMBERS: never[] = [];

export function MemberSidebar() {
  const selectedGuildId = useNavigationStore((s) => s.selectedGuildId);
  const members = useGuildStore((s) =>
    selectedGuildId ? (s.members[selectedGuildId] ?? EMPTY_MEMBERS) : EMPTY_MEMBERS,
  );

  const founders = members.filter((m) => m.role === "founder");
  const moderators = members.filter((m) => m.role === "moderator");
  const users = members.filter((m) => m.role === "user");
  const observers = members.filter((m) => m.role === "observer");

  const renderSection = (title: string, list: typeof members) => {
    if (list.length === 0) return null;
    return (
      <div className="mb-4">
        <h3 className="mb-1 px-2 text-xs font-semibold uppercase text-discord-muted">
          {title} — {list.length}
        </h3>
        {list.map((member) => {
          const isOnline = member.status !== "offline";
          return (
            <div
              key={member.peer_id}
              className="group flex items-center gap-2.5 rounded-md px-2 py-1 hover:bg-discord-hover"
            >
              <div className="relative">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-discord-blurple text-xs font-bold text-white">
                  {(member.name || "?")[0]?.toUpperCase()}
                </div>
                <div
                  className={`absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-discord-chat ${
                    isOnline ? "bg-discord-green" : "bg-discord-muted"
                  }`}
                />
              </div>
              <span
                className={`min-w-0 flex-1 truncate text-sm ${
                  isOnline ? "text-white" : "text-discord-muted"
                }`}
              >
                {member.name || member.public_key.slice(0, 8) + "..."}
              </span>
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div className="flex w-60 flex-col bg-discord-sidebar">
      <div className="flex h-12 items-center px-4">
        <h3 className="text-sm font-semibold text-discord-muted">Members</h3>
      </div>
      <div className="flex-1 overflow-y-auto px-2 py-3">
        {members.length === 0 ? (
          <p className="px-2 text-xs text-discord-muted">
            No members loaded
          </p>
        ) : (
          <>
            {renderSection("Founder", founders)}
            {renderSection("Moderators", moderators)}
            {renderSection("Members", users)}
            {renderSection("Observers", observers)}
          </>
        )}
      </div>
    </div>
  );
}
