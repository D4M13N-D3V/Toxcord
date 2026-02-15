import { useCallStore } from "../../stores/callStore";
import { LocalPreview } from "./LocalPreview";
import { RemoteVideo } from "./RemoteVideo";
import { DevicePicker, SettingsIcon } from "../call/DevicePicker";
import { useState } from "react";

/**
 * Fullscreen video modal for viewing video calls at full size.
 * Useful for screen sharing where you need to see details.
 */
export function FullscreenVideoModal() {
  const activeCall = useCallStore((s) => s.activeCall);
  const isFullscreen = useCallStore((s) => s.isFullscreen);
  const isMuted = useCallStore((s) => s.isMuted);
  const isDeafened = useCallStore((s) => s.isDeafened);
  const isScreenSharing = useCallStore((s) => s.isScreenSharing);
  const toggleMute = useCallStore((s) => s.toggleMute);
  const toggleDeafen = useCallStore((s) => s.toggleDeafen);
  const toggleVideo = useCallStore((s) => s.toggleVideo);
  const toggleScreenShare = useCallStore((s) => s.toggleScreenShare);
  const toggleFullscreen = useCallStore((s) => s.toggleFullscreen);
  const hangup = useCallStore((s) => s.hangup);
  const selectedMicId = useCallStore((s) => s.selectedMicId);
  const selectedSpeakerId = useCallStore((s) => s.selectedSpeakerId);
  const selectedCameraId = useCallStore((s) => s.selectedCameraId);
  const setSelectedMic = useCallStore((s) => s.setSelectedMic);
  const setSelectedSpeaker = useCallStore((s) => s.setSelectedSpeaker);
  const setSelectedCamera = useCallStore((s) => s.setSelectedCamera);

  const [showDevicePicker, setShowDevicePicker] = useState(false);

  if (!isFullscreen || !activeCall || activeCall.status !== "in_progress") {
    return null;
  }

  const isVideoEnabled = activeCall.hasVideo && !activeCall.isVideoMuted;

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-[#1e1f22]">
      {/* Header with close button */}
      <div className="flex h-12 items-center justify-between border-b border-[#3f4147] px-4">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-white">
            Call with {activeCall.friendName}
          </span>
          {isScreenSharing && (
            <span className="rounded bg-discord-red/20 px-2 py-0.5 text-xs text-discord-red">
              Sharing Screen
            </span>
          )}
        </div>
        <button
          onClick={() => toggleFullscreen()}
          className="flex h-8 w-8 items-center justify-center rounded-md text-discord-muted transition-colors hover:bg-discord-hover hover:text-white"
          title="Exit fullscreen"
        >
          <MinimizeIcon className="h-5 w-5" />
        </button>
      </div>

      {/* Video area - fills remaining space */}
      <div className="flex flex-1 items-center justify-center gap-6 p-6">
        {/* Remote participant - larger */}
        <div className="flex flex-col items-center">
          <div className="relative overflow-hidden rounded-xl bg-[#2b2d31]">
            {isVideoEnabled ? (
              <RemoteVideo
                friendNumber={activeCall.friendNumber}
                className="h-[70vh] max-h-[800px] w-auto max-w-[70vw] object-contain"
              />
            ) : (
              <div className="flex h-[50vh] w-[50vh] max-h-[500px] max-w-[500px] items-center justify-center">
                <div className="flex h-32 w-32 items-center justify-center rounded-full bg-discord-blurple">
                  <span className="text-5xl font-bold text-white">
                    {activeCall.friendName[0]?.toUpperCase()}
                  </span>
                </div>
              </div>
            )}
          </div>
          <span className="mt-3 text-base font-medium text-white">
            {activeCall.friendName}
          </span>
        </div>

        {/* Local participant - smaller, positioned in corner */}
        <div className="absolute bottom-24 right-6 flex flex-col items-center">
          <div className="relative overflow-hidden rounded-lg bg-[#2b2d31] shadow-xl">
            {isVideoEnabled ? (
              <LocalPreview className="h-40 w-56 object-cover" />
            ) : (
              <div className="flex h-40 w-56 items-center justify-center">
                <div className="flex h-16 w-16 items-center justify-center rounded-full bg-discord-green">
                  <span className="text-xl font-bold text-white">You</span>
                </div>
              </div>
            )}
            {isMuted && (
              <div className="absolute bottom-2 right-2 rounded-full bg-discord-red p-1.5">
                <MicOffIcon className="h-4 w-4 text-white" />
              </div>
            )}
          </div>
          <span className="mt-2 text-sm font-medium text-discord-muted">You</span>
        </div>
      </div>

      {/* Controls bar */}
      <div className="relative flex items-center justify-center gap-3 border-t border-[#3f4147] bg-[#232428] p-4">
        {/* Mute */}
        <button
          onClick={() => toggleMute()}
          className={`flex h-12 w-12 items-center justify-center rounded-full transition-colors ${
            isMuted
              ? "bg-[#ed4245] hover:bg-[#ed4245]/80"
              : "bg-[#3c3f45] hover:bg-[#4e5058]"
          }`}
          title={isMuted ? "Unmute" : "Mute"}
        >
          {isMuted ? (
            <MicOffIcon className="h-6 w-6 text-white" />
          ) : (
            <MicIcon className="h-6 w-6 text-white" />
          )}
        </button>

        {/* Video toggle */}
        <button
          onClick={() => toggleVideo()}
          className={`flex h-12 w-12 items-center justify-center rounded-full transition-colors ${
            !isVideoEnabled
              ? "bg-[#ed4245] hover:bg-[#ed4245]/80"
              : "bg-[#3c3f45] hover:bg-[#4e5058]"
          }`}
          title={isVideoEnabled ? "Turn off camera" : "Turn on camera"}
        >
          {isVideoEnabled ? (
            <VideoIcon className="h-6 w-6 text-white" />
          ) : (
            <VideoOffIcon className="h-6 w-6 text-white" />
          )}
        </button>

        {/* Screen share */}
        <button
          onClick={() => toggleScreenShare()}
          className={`flex h-12 w-12 items-center justify-center rounded-full transition-colors ${
            isScreenSharing
              ? "bg-[#ed4245] hover:bg-[#ed4245]/80"
              : "bg-[#3c3f45] hover:bg-[#4e5058]"
          }`}
          title={isScreenSharing ? "Stop Sharing" : "Share Your Screen"}
        >
          <ScreenShareIcon className="h-6 w-6 text-white" />
        </button>

        {/* Deafen */}
        <button
          onClick={() => toggleDeafen()}
          className={`flex h-12 w-12 items-center justify-center rounded-full transition-colors ${
            isDeafened
              ? "bg-[#ed4245] hover:bg-[#ed4245]/80"
              : "bg-[#3c3f45] hover:bg-[#4e5058]"
          }`}
          title={isDeafened ? "Undeafen" : "Deafen"}
        >
          {isDeafened ? (
            <HeadphoneOffIcon className="h-6 w-6 text-white" />
          ) : (
            <HeadphoneIcon className="h-6 w-6 text-white" />
          )}
        </button>

        {/* Settings */}
        <button
          onClick={() => setShowDevicePicker(!showDevicePicker)}
          className={`flex h-12 w-12 items-center justify-center rounded-full transition-colors ${
            showDevicePicker
              ? "bg-discord-blurple hover:bg-discord-blurple/80"
              : "bg-[#3c3f45] hover:bg-[#4e5058]"
          }`}
          title="Device settings"
        >
          <SettingsIcon className="h-6 w-6 text-white" />
        </button>

        {/* Hangup */}
        <button
          onClick={() => hangup()}
          className="flex h-12 w-24 items-center justify-center rounded-full bg-[#ed4245] transition-colors hover:bg-[#ed4245]/80"
          title="End call"
        >
          <PhoneOffIcon className="h-6 w-6 text-white" />
        </button>

        {/* Device Picker Dropdown */}
        {showDevicePicker && (
          <div className="absolute bottom-20">
            <DevicePicker
              onClose={() => setShowDevicePicker(false)}
              selectedMicId={selectedMicId}
              selectedSpeakerId={selectedSpeakerId}
              selectedCameraId={selectedCameraId}
              onSelectMic={setSelectedMic}
              onSelectSpeaker={setSelectedSpeaker}
              onSelectCamera={setSelectedCamera}
            />
          </div>
        )}
      </div>
    </div>
  );
}

// Icons
function MinimizeIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
    </svg>
  );
}

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

function PhoneOffIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M12 9c-1.6 0-3.15.25-4.6.72v3.1c0 .39-.23.74-.56.9-.98.49-1.87 1.12-2.66 1.85-.18.18-.43.28-.7.28-.28 0-.53-.11-.71-.29L.29 13.08c-.18-.17-.29-.42-.29-.7 0-.28.11-.53.29-.71C3.34 8.78 7.46 7 12 7s8.66 1.78 11.71 4.67c.18.18.29.43.29.71 0 .28-.11.53-.29.71l-2.48 2.48c-.18.18-.43.29-.71.29-.27 0-.52-.11-.7-.28-.79-.74-1.68-1.36-2.66-1.85-.33-.16-.56-.5-.56-.9v-3.1C15.15 9.25 13.6 9 12 9z" />
    </svg>
  );
}

function ScreenShareIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="currentColor" viewBox="0 0 24 24">
      <path d="M20 18c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2H0v2h24v-2h-4zM4 6h16v10H4V6z" />
    </svg>
  );
}
