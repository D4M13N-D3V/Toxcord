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

export interface VideoDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export interface VideoFramePayload {
  friend_number: number;
  width: number;
  height: number;
  data: number[];
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
    }
  | {
      type: "VideoFrame";
      data: VideoFramePayload;
    }
  | {
      type: "VideoError";
      data: { error: string };
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

// ─── Video Devices ───────────────────────────────────────────────────

export async function listVideoDevices(): Promise<VideoDevice[]> {
  return invoke("list_video_devices");
}

// ─── Device Selection ─────────────────────────────────────────────────

export async function setAudioInputDevice(deviceId: string): Promise<void> {
  return invoke("set_audio_input_device", { deviceId });
}

export async function setAudioOutputDevice(deviceId: string): Promise<void> {
  return invoke("set_audio_output_device", { deviceId });
}

export async function setVideoDevice(deviceId: string): Promise<void> {
  return invoke("set_video_device", { deviceId });
}

// ─── Camera Diagnostics ───────────────────────────────────────────────

export interface CameraStatus {
  has_usb_camera: boolean;
  has_video_device: boolean;
  needs_driver_load: boolean;
  usb_camera_name: string | null;
}

export async function checkCameraStatus(): Promise<CameraStatus> {
  return invoke("check_camera_status");
}

export async function loadCameraDriver(): Promise<void> {
  return invoke("load_camera_driver");
}

// ─── Event Listening ─────────────────────────────────────────────────

export function onToxAvEvent(
  callback: (event: ToxAvEvent) => void,
): Promise<UnlistenFn> {
  return listen<ToxAvEvent>("toxav://event", (event) => {
    callback(event.payload);
  });
}

export function onLocalVideoFrame(
  callback: (frame: VideoFramePayload) => void,
): Promise<UnlistenFn> {
  return listen<{ type: "VideoFrame"; data: VideoFramePayload }>(
    "toxav://local-video",
    (event) => {
      callback(event.payload.data);
    },
  );
}

export function onRemoteVideoFrame(
  callback: (frame: VideoFramePayload) => void,
): Promise<UnlistenFn> {
  return listen<ToxAvEvent>("toxav://event", (event) => {
    if (event.payload.type === "VideoFrame") {
      callback(event.payload.data);
    }
  });
}
