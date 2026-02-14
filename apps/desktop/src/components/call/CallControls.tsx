import { useCallStore } from "../../stores/callStore";

interface CallControlsProps {
  minimal?: boolean;
}

export function CallControls({ minimal = false }: CallControlsProps) {
  const activeCall = useCallStore((s) => s.activeCall);
  const isMuted = useCallStore((s) => s.isMuted);
  const isDeafened = useCallStore((s) => s.isDeafened);
  const toggleMute = useCallStore((s) => s.toggleMute);
  const toggleDeafen = useCallStore((s) => s.toggleDeafen);
  const toggleVideo = useCallStore((s) => s.toggleVideo);
  const hangup = useCallStore((s) => s.hangup);

  if (!activeCall) return null;

  return (
    <div className={`flex items-center ${minimal ? "gap-2" : "gap-3"}`}>
      {/* Mute button */}
      <button
        onClick={() => toggleMute()}
        className={`flex items-center justify-center rounded-full transition-colors ${
          minimal ? "h-8 w-8" : "h-10 w-10"
        } ${
          isMuted
            ? "bg-discord-red hover:bg-discord-red/80"
            : "bg-discord-sidebar hover:bg-discord-hover"
        }`}
        title={isMuted ? "Unmute" : "Mute"}
      >
        {isMuted ? (
          <MicOffIcon className={minimal ? "h-4 w-4" : "h-5 w-5"} />
        ) : (
          <MicIcon className={minimal ? "h-4 w-4" : "h-5 w-5"} />
        )}
      </button>

      {/* Deafen button */}
      <button
        onClick={toggleDeafen}
        className={`flex items-center justify-center rounded-full transition-colors ${
          minimal ? "h-8 w-8" : "h-10 w-10"
        } ${
          isDeafened
            ? "bg-discord-red hover:bg-discord-red/80"
            : "bg-discord-sidebar hover:bg-discord-hover"
        }`}
        title={isDeafened ? "Undeafen" : "Deafen"}
      >
        {isDeafened ? (
          <HeadphoneOffIcon className={minimal ? "h-4 w-4" : "h-5 w-5"} />
        ) : (
          <HeadphoneIcon className={minimal ? "h-4 w-4" : "h-5 w-5"} />
        )}
      </button>

      {/* Video button (if video call) */}
      {activeCall.hasVideo && (
        <button
          onClick={() => toggleVideo()}
          className={`flex items-center justify-center rounded-full transition-colors ${
            minimal ? "h-8 w-8" : "h-10 w-10"
          } ${
            activeCall.isVideoMuted
              ? "bg-discord-red hover:bg-discord-red/80"
              : "bg-discord-sidebar hover:bg-discord-hover"
          }`}
          title={activeCall.isVideoMuted ? "Turn on camera" : "Turn off camera"}
        >
          {activeCall.isVideoMuted ? (
            <VideoOffIcon className={minimal ? "h-4 w-4" : "h-5 w-5"} />
          ) : (
            <VideoIcon className={minimal ? "h-4 w-4" : "h-5 w-5"} />
          )}
        </button>
      )}

      {/* Hangup button */}
      <button
        onClick={() => hangup()}
        className={`flex items-center justify-center rounded-full bg-discord-red transition-colors hover:bg-discord-red/80 ${
          minimal ? "h-8 w-8" : "h-10 w-10"
        }`}
        title="End call"
      >
        <PhoneOffIcon className={minimal ? "h-4 w-4" : "h-5 w-5"} />
      </button>
    </div>
  );
}

// Icons
function MicIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
      <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
    </svg>
  );
}

function MicOffIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M19 11h-1.7c0 .74-.16 1.43-.43 2.05l1.23 1.23c.56-.98.9-2.09.9-3.28zm-4.02.17c0-.06.02-.11.02-.17V5c0-1.66-1.34-3-3-3S9 3.34 9 5v.18l5.98 5.99zM4.27 3L3 4.27l6.01 6.01V11c0 1.66 1.33 3 2.99 3 .22 0 .44-.03.65-.08l1.66 1.66c-.71.33-1.5.52-2.31.52-2.76 0-5.3-2.1-5.3-5.1H5c0 3.41 2.72 6.23 6 6.72V21h2v-3.28c.91-.13 1.77-.45 2.54-.9L19.73 21 21 19.73 4.27 3z" />
    </svg>
  );
}

function HeadphoneIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M12 1c-4.97 0-9 4.03-9 9v7c0 1.66 1.34 3 3 3h3v-8H5v-2c0-3.87 3.13-7 7-7s7 3.13 7 7v2h-4v8h3c1.66 0 3-1.34 3-3v-7c0-4.97-4.03-9-9-9z" />
    </svg>
  );
}

function HeadphoneOffIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M12 1c-4.97 0-9 4.03-9 9v7c0 1.66 1.34 3 3 3h3v-8H5v-2c0-3.87 3.13-7 7-7s7 3.13 7 7v2h-4v8h3c1.66 0 3-1.34 3-3v-7c0-4.97-4.03-9-9-9z" />
      <line x1="3" y1="3" x2="21" y2="21" stroke="currentColor" strokeWidth="2" />
    </svg>
  );
}

function VideoIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z" />
    </svg>
  );
}

function VideoOffIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M21 6.5l-4 4V7c0-.55-.45-1-1-1H9.82L21 17.18V6.5zM3.27 2L2 3.27 4.73 6H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.21 0 .39-.08.54-.18L19.73 21 21 19.73 3.27 2z" />
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
