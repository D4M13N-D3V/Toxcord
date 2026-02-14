import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// ─── Types ───────────────────────────────────────────────────────────

export type CallStatus =
  | "ringing_outgoing"
  | "ringing_incoming"
  | "in_progress"
  | "ended"
  | "error";

export interface CallState {
  friend_number: number;
  state: CallStatus;
  has_audio: boolean;
  has_video: boolean;
  is_audio_muted: boolean;
  is_video_muted: boolean;
  started_at: string | null;
}

export interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export type ToxAvEvent =
  | {
      type: "IncomingCall";
      data: { friend_number: number; audio_enabled: boolean; video_enabled: boolean };
    }
  | {
      type: "CallStateChange";
      data: {
        friend_number: number;
        state: string;
        sending_audio: boolean;
        sending_video: boolean;
        accepting_audio: boolean;
        accepting_video: boolean;
      };
    }
  | {
      type: "CallEnded";
      data: { friend_number: number; reason: string };
    }
  | {
      type: "AudioLevelUpdate";
      data: { friend_number: number; level: number };
    };

// ─── Call Management ─────────────────────────────────────────────────

export async function callFriend(
  friendNumber: number,
  withVideo: boolean = false,
): Promise<void> {
  return invoke("call_friend", { friendNumber, withVideo });
}

export async function answerCall(
  friendNumber: number,
  withVideo: boolean = false,
): Promise<void> {
  return invoke("answer_call", { friendNumber, withVideo });
}

export async function hangupCall(friendNumber: number): Promise<void> {
  return invoke("hangup_call", { friendNumber });
}

export async function toggleMute(
  friendNumber: number,
  muted: boolean,
): Promise<void> {
  return invoke("toggle_mute", { friendNumber, muted });
}

export async function toggleVideo(
  friendNumber: number,
  enabled: boolean,
): Promise<void> {
  return invoke("toggle_video", { friendNumber, enabled });
}

export async function getCallState(
  friendNumber: number,
): Promise<CallState | null> {
  return invoke("get_call_state", { friendNumber });
}

// ─── Audio Devices ───────────────────────────────────────────────────

export async function listAudioInputDevices(): Promise<AudioDevice[]> {
  return invoke("list_audio_input_devices");
}

export async function listAudioOutputDevices(): Promise<AudioDevice[]> {
  return invoke("list_audio_output_devices");
}

// ─── Event Listening ─────────────────────────────────────────────────

export function onToxAvEvent(
  callback: (event: ToxAvEvent) => void,
): Promise<UnlistenFn> {
  return listen<ToxAvEvent>("toxav://event", (event) => {
    callback(event.payload);
  });
}
