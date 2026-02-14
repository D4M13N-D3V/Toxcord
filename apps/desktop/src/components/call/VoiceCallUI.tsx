import { useCallStore } from "../../stores/callStore";
import { CallControls } from "./CallControls";

export function VoiceCallUI() {
  const activeCall = useCallStore((s) => s.activeCall);

  if (!activeCall || activeCall.status !== "in_progress") {
    return null;
  }

  return (
    <div className="flex flex-col items-center justify-center bg-discord-chat p-8">
      {/* Participant avatar */}
      <div className="mb-6 flex h-32 w-32 items-center justify-center rounded-full bg-discord-blurple">
        <span className="text-5xl font-bold text-white">
          {activeCall.friendName[0]?.toUpperCase()}
        </span>
      </div>

      <h2 className="mb-2 text-xl font-semibold text-white">
        {activeCall.friendName}
      </h2>

      <p className="mb-8 text-sm text-discord-green">
        {formatDuration(activeCall.duration)}
      </p>

      <CallControls />
    </div>
  );
}

/** Mini call indicator for showing in DM header during active call */
export function MiniCallIndicator() {
  const activeCall = useCallStore((s) => s.activeCall);
  const isMuted = useCallStore((s) => s.isMuted);
  const hangup = useCallStore((s) => s.hangup);

  if (!activeCall || activeCall.status !== "in_progress") {
    return null;
  }

  return (
    <div className="flex items-center gap-3 rounded-lg bg-discord-green/20 px-3 py-2">
      {/* Call info */}
      <div className="flex items-center gap-2">
        <div className="h-2 w-2 animate-pulse rounded-full bg-discord-green" />
        <span className="text-sm font-medium text-discord-green">
          {formatDuration(activeCall.duration)}
        </span>
      </div>

      {/* Mute indicator */}
      {isMuted && (
        <div className="text-discord-red">
          <MicOffIcon className="h-4 w-4" />
        </div>
      )}

      {/* Hangup button */}
      <button
        onClick={() => hangup()}
        className="flex h-6 w-6 items-center justify-center rounded-full bg-discord-red transition-colors hover:bg-discord-red/80"
        title="End call"
      >
        <PhoneOffIcon className="h-3 w-3 text-white" />
      </button>
    </div>
  );
}

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

// Icons
function MicOffIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M19 11h-1.7c0 .74-.16 1.43-.43 2.05l1.23 1.23c.56-.98.9-2.09.9-3.28zm-4.02.17c0-.06.02-.11.02-.17V5c0-1.66-1.34-3-3-3S9 3.34 9 5v.18l5.98 5.99zM4.27 3L3 4.27l6.01 6.01V11c0 1.66 1.33 3 2.99 3 .22 0 .44-.03.65-.08l1.66 1.66c-.71.33-1.5.52-2.31.52-2.76 0-5.3-2.1-5.3-5.1H5c0 3.41 2.72 6.23 6 6.72V21h2v-3.28c.91-.13 1.77-.45 2.54-.9L19.73 21 21 19.73 4.27 3z" />
    </svg>
  );
}

function PhoneOffIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M12 9c-1.6 0-3.15.25-4.6.72v3.1c0 .39-.23.74-.56.9-.98.49-1.87 1.12-2.66 1.85-.18.18-.43.28-.7.28-.28 0-.53-.11-.71-.29L.29 13.08c-.18-.17-.29-.42-.29-.7 0-.28.11-.53.29-.71C3.34 8.78 7.46 7 12 7s8.66 1.78 11.71 4.67c.18.18.29.43.29.71 0 .28-.11.53-.29.71l-2.48 2.48c-.18.18-.43.29-.71.29-.27 0-.52-.11-.7-.28-.79-.74-1.68-1.36-2.66-1.85-.33-.16-.56-.5-.56-.9v-3.1C15.15 9.25 13.6 9 12 9z" />
      <line x1="3" y1="3" x2="21" y2="21" stroke="currentColor" strokeWidth="2" />
    </svg>
  );
}
