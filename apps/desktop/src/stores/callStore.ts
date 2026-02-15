import { create } from "zustand";
import * as api from "../api/calls";
import type { CallStatus, ScreenInfo } from "../api/calls";

interface ActiveCall {
  friendNumber: number;
  friendName: string;
  status: CallStatus;
  hasAudio: boolean;
  hasVideo: boolean;
  isAudioMuted: boolean;
  isVideoMuted: boolean;
  startedAt: string | null;
  /** Duration in seconds (updated while call is active) */
  duration: number;
}

interface IncomingCall {
  friendNumber: number;
  friendName: string;
  audioEnabled: boolean;
  videoEnabled: boolean;
  receivedAt: number;
}

interface CallStoreState {
  /** Current active call (if any) */
  activeCall: ActiveCall | null;
  /** Pending incoming call (if any) */
  incomingCall: IncomingCall | null;
  /** Whether we're currently connecting */
  isConnecting: boolean;
  /** Whether microphone is muted */
  isMuted: boolean;
  /** Whether speaker is deafened */
  isDeafened: boolean;
  /** Whether screen sharing is active */
  isScreenSharing: boolean;
  /** Available screens for sharing */
  availableScreens: ScreenInfo[];
  /** Whether video is in fullscreen mode */
  isFullscreen: boolean;

  /** Selected device IDs */
  selectedMicId: string | null;
  selectedSpeakerId: string | null;
  selectedCameraId: string | null;

  // Actions
  startCall: (friendNumber: number, friendName: string, withVideo?: boolean) => Promise<void>;
  answerCall: (withVideo?: boolean) => Promise<void>;
  declineCall: () => Promise<void>;
  hangup: () => Promise<void>;
  toggleMute: () => Promise<void>;
  toggleDeafen: () => void;
  toggleVideo: () => Promise<void>;
  toggleScreenShare: () => Promise<void>;
  toggleFullscreen: () => void;

  // Device selection
  setSelectedMic: (id: string) => Promise<void>;
  setSelectedSpeaker: (id: string) => Promise<void>;
  setSelectedCamera: (id: string) => Promise<void>;

  // Internal state updates from events
  setIncomingCall: (call: IncomingCall | null) => void;
  updateCallState: (friendNumber: number, state: CallStatus, flags: {
    sendingAudio: boolean;
    sendingVideo: boolean;
    acceptingAudio: boolean;
    acceptingVideo: boolean;
  }) => void;
  endCall: (friendNumber: number, reason: string) => void;
  updateDuration: () => void;
}

export const useCallStore = create<CallStoreState>((set, get) => ({
  activeCall: null,
  incomingCall: null,
  isConnecting: false,
  isMuted: false,
  isDeafened: false,
  isScreenSharing: false,
  availableScreens: [],
  isFullscreen: false,
  selectedMicId: null,
  selectedSpeakerId: null,
  selectedCameraId: null,

  startCall: async (friendNumber, friendName, withVideo = false) => {
    set({ isConnecting: true });
    try {
      await api.callFriend(friendNumber, withVideo);
      set({
        activeCall: {
          friendNumber,
          friendName,
          status: "ringing_outgoing",
          hasAudio: true,
          hasVideo: withVideo,
          isAudioMuted: false,
          isVideoMuted: !withVideo,
          startedAt: null,
          duration: 0,
        },
        isConnecting: false,
      });
    } catch (e) {
      console.error("Failed to start call:", e);
      set({ isConnecting: false });
    }
  },

  answerCall: async (withVideo = false) => {
    const { incomingCall } = get();
    if (!incomingCall) return;

    set({ isConnecting: true });
    try {
      await api.answerCall(incomingCall.friendNumber, withVideo);
      set({
        activeCall: {
          friendNumber: incomingCall.friendNumber,
          friendName: incomingCall.friendName,
          status: "in_progress",
          hasAudio: incomingCall.audioEnabled,
          hasVideo: withVideo || incomingCall.videoEnabled,
          isAudioMuted: false,
          isVideoMuted: !withVideo,
          startedAt: new Date().toISOString(),
          duration: 0,
        },
        incomingCall: null,
        isConnecting: false,
      });
    } catch (e) {
      console.error("Failed to answer call:", e);
      set({ isConnecting: false });
    }
  },

  declineCall: async () => {
    const { incomingCall } = get();
    if (!incomingCall) return;

    try {
      await api.hangupCall(incomingCall.friendNumber);
      set({ incomingCall: null });
    } catch (e) {
      console.error("Failed to decline call:", e);
    }
  },

  hangup: async () => {
    const { activeCall } = get();
    if (!activeCall) return;

    try {
      await api.hangupCall(activeCall.friendNumber);
      set({ activeCall: null, isMuted: false, isDeafened: false, isScreenSharing: false, isFullscreen: false });
    } catch (e) {
      console.error("Failed to hangup:", e);
    }
  },

  toggleMute: async () => {
    const { activeCall, isMuted } = get();
    if (!activeCall) return;

    const newMuted = !isMuted;
    try {
      await api.toggleMute(activeCall.friendNumber, newMuted);
      set({ isMuted: newMuted });
    } catch (e) {
      console.error("Failed to toggle mute:", e);
    }
  },

  toggleDeafen: () => {
    set((s) => ({ isDeafened: !s.isDeafened }));
  },

  setSelectedMic: async (id) => {
    set({ selectedMicId: id });
    console.log("[CallStore] Selected microphone:", id);
    try {
      await api.setAudioInputDevice(id);
    } catch (e) {
      console.error("Failed to set audio input device:", e);
    }
  },

  setSelectedSpeaker: async (id) => {
    set({ selectedSpeakerId: id });
    console.log("[CallStore] Selected speaker:", id);
    try {
      await api.setAudioOutputDevice(id);
    } catch (e) {
      console.error("Failed to set audio output device:", e);
    }
  },

  setSelectedCamera: async (id) => {
    set({ selectedCameraId: id });
    console.log("[CallStore] Selected camera:", id);
    try {
      await api.setVideoDevice(id);
    } catch (e) {
      console.error("Failed to set video device:", e);
    }
  },

  toggleVideo: async () => {
    const { activeCall } = get();
    if (!activeCall) return;

    const newEnabled = !activeCall.hasVideo;
    try {
      await api.toggleVideo(activeCall.friendNumber, newEnabled);
      set({
        activeCall: {
          ...activeCall,
          hasVideo: newEnabled,
          isVideoMuted: !newEnabled,
        },
      });
    } catch (e) {
      console.error("Failed to toggle video:", e);
    }
  },

  toggleScreenShare: async () => {
    const { activeCall, isScreenSharing } = get();
    if (!activeCall) return;

    try {
      if (isScreenSharing) {
        // Stop screen sharing
        await api.stopScreenShare();
        set({ isScreenSharing: false });
        console.log("[CallStore] Screen sharing stopped");
      } else {
        // Start screen sharing - get screens and share primary
        const screens = await api.listScreens();
        console.log("[CallStore] Available screens:", screens);

        const primary = screens.find((s) => s.is_primary) || screens[0];
        if (primary) {
          await api.startScreenShare(primary.id);
          set({ isScreenSharing: true, availableScreens: screens });
          console.log("[CallStore] Screen sharing started:", primary.name);
        } else {
          console.error("[CallStore] No screens available for sharing");
        }
      }
    } catch (e) {
      console.error("Failed to toggle screen share:", e);
    }
  },

  toggleFullscreen: () => {
    set((s) => ({ isFullscreen: !s.isFullscreen }));
  },

  setIncomingCall: (call) => {
    set({ incomingCall: call });
  },

  updateCallState: (friendNumber, status, flags) => {
    set((s) => {
      // If we have an active call with this friend, update it
      if (s.activeCall?.friendNumber === friendNumber) {
        const isNowInProgress = status === "in_progress" && s.activeCall.status !== "in_progress";
        return {
          activeCall: {
            ...s.activeCall,
            status,
            hasAudio: flags.sendingAudio || flags.acceptingAudio,
            hasVideo: flags.sendingVideo || flags.acceptingVideo,
            startedAt: isNowInProgress ? new Date().toISOString() : s.activeCall.startedAt,
          },
        };
      }
      return s;
    });
  },

  endCall: (friendNumber, _reason) => {
    set((s) => {
      if (s.activeCall?.friendNumber === friendNumber) {
        return { activeCall: null, isMuted: false, isDeafened: false, isScreenSharing: false, isFullscreen: false };
      }
      if (s.incomingCall?.friendNumber === friendNumber) {
        return { incomingCall: null };
      }
      return s;
    });
  },

  updateDuration: () => {
    set((s) => {
      if (s.activeCall?.status === "in_progress" && s.activeCall.startedAt) {
        const start = new Date(s.activeCall.startedAt).getTime();
        const duration = Math.floor((Date.now() - start) / 1000);
        return {
          activeCall: { ...s.activeCall, duration },
        };
      }
      return s;
    });
  },
}));
