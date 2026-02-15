import { useState, useEffect, useRef } from "react";
import {
  listAudioInputDevices,
  listAudioOutputDevices,
  listVideoDevices,
  checkCameraStatus,
  loadCameraDriver,
  AudioDevice,
  VideoDevice,
  CameraStatus,
} from "../../api/calls";

interface DevicePickerProps {
  onClose: () => void;
  selectedMicId: string | null;
  selectedSpeakerId: string | null;
  selectedCameraId: string | null;
  onSelectMic: (id: string) => void;
  onSelectSpeaker: (id: string) => void;
  onSelectCamera: (id: string) => void;
}

export function DevicePicker({
  onClose,
  selectedMicId,
  selectedSpeakerId,
  selectedCameraId,
  onSelectMic,
  onSelectSpeaker,
  onSelectCamera,
}: DevicePickerProps) {
  const [mics, setMics] = useState<AudioDevice[]>([]);
  const [speakers, setSpeakers] = useState<AudioDevice[]>([]);
  const [cameras, setCameras] = useState<VideoDevice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [cameraStatus, setCameraStatus] = useState<CameraStatus | null>(null);
  const [loadingDriver, setLoadingDriver] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const loadDevices = async () => {
    setLoading(true);
    setError(null);
    try {
      const [micList, speakerList, cameraList, camStatus] = await Promise.all([
        listAudioInputDevices(),
        listAudioOutputDevices(),
        listVideoDevices(),
        checkCameraStatus(),
      ]);
      console.log("[DevicePicker] Loaded devices:", { micList, speakerList, cameraList, camStatus });
      setMics(micList);
      setSpeakers(speakerList);
      setCameras(cameraList);
      setCameraStatus(camStatus);
    } catch (e) {
      console.error("[DevicePicker] Failed to load devices:", e);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const [driverError, setDriverError] = useState<string | null>(null);

  const handleLoadDriver = async () => {
    setLoadingDriver(true);
    setDriverError(null);
    try {
      await loadCameraDriver();
      // Reload devices after driver is loaded
      await loadDevices();
    } catch (e) {
      console.error("[DevicePicker] Failed to load driver:", e);
      const errStr = String(e);
      if (errStr.includes("not found in directory")) {
        setDriverError("Kernel modules out of sync. Please reboot your system.");
      } else if (errStr.includes("dismissed") || errStr.includes("cancelled")) {
        setDriverError(null); // User cancelled, not an error
      } else {
        setDriverError(errStr);
      }
    } finally {
      setLoadingDriver(false);
    }
  };

  useEffect(() => {
    loadDevices();
  }, []);

  // Close on click outside
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [onClose]);

  return (
    <div
      ref={ref}
      className="absolute right-0 top-full z-50 mt-2 w-72 rounded-lg bg-[#2b2d31] p-3 shadow-xl border border-[#3f4147]"
    >
      <h3 className="mb-3 text-sm font-semibold text-white">Device Settings</h3>

      {loading && (
        <div className="flex items-center justify-center py-4">
          <div className="h-5 w-5 animate-spin rounded-full border-2 border-white border-t-transparent" />
        </div>
      )}

      {error && (
        <div className="rounded bg-discord-red/20 p-2 text-xs text-discord-red">
          {error}
        </div>
      )}

      {!loading && !error && (
        <div className="space-y-3">
          {/* Microphone */}
          <div>
            <label className="mb-1 block text-xs font-medium text-discord-muted">
              Microphone
            </label>
            <select
              value={selectedMicId || ""}
              onChange={(e) => onSelectMic(e.target.value)}
              className="w-full rounded bg-[#383a40] px-2 py-1.5 text-sm text-white outline-none focus:ring-1 focus:ring-discord-blurple [&>option]:bg-[#383a40] [&>option]:text-white"
            >
              {mics.length === 0 ? (
                <option value="">No microphones found</option>
              ) : (
                mics.map((mic) => (
                  <option key={mic.id} value={mic.id}>
                    {mic.name} {mic.is_default ? "(Default)" : ""}
                  </option>
                ))
              )}
            </select>
          </div>

          {/* Speaker */}
          <div>
            <label className="mb-1 block text-xs font-medium text-discord-muted">
              Speaker
            </label>
            <select
              value={selectedSpeakerId || ""}
              onChange={(e) => onSelectSpeaker(e.target.value)}
              className="w-full rounded bg-[#383a40] px-2 py-1.5 text-sm text-white outline-none focus:ring-1 focus:ring-discord-blurple [&>option]:bg-[#383a40] [&>option]:text-white"
            >
              {speakers.length === 0 ? (
                <option value="">No speakers found</option>
              ) : (
                speakers.map((speaker) => (
                  <option key={speaker.id} value={speaker.id}>
                    {speaker.name} {speaker.is_default ? "(Default)" : ""}
                  </option>
                ))
              )}
            </select>
          </div>

          {/* Camera */}
          <div>
            <label className="mb-1 block text-xs font-medium text-discord-muted">
              Camera
            </label>
            {cameras.length === 0 && cameraStatus?.needs_driver_load ? (
              <div className="rounded bg-[#383a40] p-2">
                {driverError ? (
                  <>
                    <p className="text-xs text-discord-red mb-2">{driverError}</p>
                    <button
                      onClick={handleLoadDriver}
                      disabled={loadingDriver}
                      className="w-full rounded bg-[#4e5058] px-2 py-1.5 text-xs text-white hover:bg-[#5d6068] disabled:opacity-50"
                    >
                      Try Again
                    </button>
                  </>
                ) : (
                  <>
                    <p className="text-xs text-discord-muted mb-2">
                      Camera detected{cameraStatus.usb_camera_name ? ` (${cameraStatus.usb_camera_name})` : ""} but driver not loaded
                    </p>
                    <button
                      onClick={handleLoadDriver}
                      disabled={loadingDriver}
                      className="w-full rounded bg-discord-blurple px-2 py-1.5 text-xs text-white hover:bg-discord-blurple/80 disabled:opacity-50"
                    >
                      {loadingDriver ? "Loading driver..." : "Enable Camera"}
                    </button>
                  </>
                )}
              </div>
            ) : (
              <select
                value={selectedCameraId || ""}
                onChange={(e) => onSelectCamera(e.target.value)}
                className="w-full rounded bg-[#383a40] px-2 py-1.5 text-sm text-white outline-none focus:ring-1 focus:ring-discord-blurple [&>option]:bg-[#383a40] [&>option]:text-white"
              >
                {cameras.length === 0 ? (
                  <option value="">No cameras found</option>
                ) : (
                  cameras.map((camera) => (
                    <option key={camera.id} value={camera.id}>
                      {camera.name} {camera.is_default ? "(Default)" : ""}
                    </option>
                  ))
                )}
              </select>
            )}
          </div>

          {/* Refresh button */}
          <button
            onClick={loadDevices}
            disabled={loading}
            className="w-full rounded bg-[#3c3f45] px-2 py-1.5 text-xs text-white hover:bg-[#4e5058] disabled:opacity-50"
          >
            {loading ? "Scanning..." : "Refresh Devices"}
          </button>

          {/* Info text */}
          <p className="text-[10px] text-discord-muted">
            Device changes will apply to your next call.
          </p>
        </div>
      )}
    </div>
  );
}

function SettingsIcon({ className }: { className?: string }) {
  return (
    <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
      />
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
      />
    </svg>
  );
}

export { SettingsIcon };
