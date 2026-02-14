import { useCallStore } from "../../stores/callStore";
import { CallControls } from "./CallControls";

export function CallOverlay() {
  const incomingCall = useCallStore((s) => s.incomingCall);
  const activeCall = useCallStore((s) => s.activeCall);
  const isConnecting = useCallStore((s) => s.isConnecting);
  const answerCall = useCallStore((s) => s.answerCall);
  const declineCall = useCallStore((s) => s.declineCall);

  // Show incoming call notification
  if (incomingCall) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70">
        <div className="flex w-80 flex-col items-center rounded-lg bg-discord-sidebar p-6">
          {/* Caller avatar */}
          <div className="mb-4 flex h-20 w-20 animate-pulse items-center justify-center rounded-full bg-discord-blurple">
            <span className="text-3xl font-bold text-white">
              {incomingCall.friendName[0]?.toUpperCase()}
            </span>
          </div>

          <h3 className="mb-1 text-lg font-semibold text-white">
            {incomingCall.friendName}
          </h3>
          <p className="mb-6 text-sm text-discord-muted">
            {incomingCall.videoEnabled ? "Video call" : "Voice call"}
          </p>

          {/* Call actions */}
          <div className="flex gap-4">
            <button
              onClick={() => declineCall()}
              className="flex h-14 w-14 items-center justify-center rounded-full bg-discord-red transition-colors hover:bg-discord-red/80"
              title="Decline"
            >
              <PhoneOffIcon className="h-6 w-6 text-white" />
            </button>
            <button
              onClick={() => answerCall(incomingCall.videoEnabled)}
              className="flex h-14 w-14 items-center justify-center rounded-full bg-discord-green transition-colors hover:bg-discord-green/80"
              title="Answer"
            >
              <PhoneIcon className="h-6 w-6 text-white" />
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Show active call overlay for ringing calls
  if (activeCall && (activeCall.status === "ringing_outgoing" || isConnecting)) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70">
        <div className="flex w-80 flex-col items-center rounded-lg bg-discord-sidebar p-6">
          {/* Callee avatar */}
          <div className="mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-discord-blurple">
            <span className="text-3xl font-bold text-white">
              {activeCall.friendName[0]?.toUpperCase()}
            </span>
          </div>

          <h3 className="mb-1 text-lg font-semibold text-white">
            {activeCall.friendName}
          </h3>
          <p className="mb-6 flex items-center gap-2 text-sm text-discord-muted">
            <span className="flex gap-0.5">
              <span
                className="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-discord-green"
                style={{ animationDelay: "0ms" }}
              />
              <span
                className="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-discord-green"
                style={{ animationDelay: "150ms" }}
              />
              <span
                className="inline-block h-1.5 w-1.5 animate-bounce rounded-full bg-discord-green"
                style={{ animationDelay: "300ms" }}
              />
            </span>
            Calling...
          </p>

          <CallControls />
        </div>
      </div>
    );
  }

  return null;
}

// Icons
function PhoneIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M20.01 15.38c-1.23 0-2.42-.2-3.53-.56-.35-.12-.74-.03-1.01.24l-1.57 1.97c-2.83-1.35-5.48-3.9-6.89-6.83l1.95-1.66c.27-.28.35-.67.24-1.02-.37-1.11-.56-2.3-.56-3.53 0-.54-.45-.99-.99-.99H4.19C3.65 3 3 3.24 3 3.99 3 13.28 10.73 21 20.01 21c.71 0 .99-.63.99-1.18v-3.45c0-.54-.45-.99-.99-.99z" />
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
