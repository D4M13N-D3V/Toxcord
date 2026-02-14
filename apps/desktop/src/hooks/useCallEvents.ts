import { useEffect, useRef } from "react";
import { onToxAvEvent, ToxAvEvent, CallStatus } from "../api/calls";
import { useCallStore } from "../stores/callStore";
import { useFriendStore } from "../stores/friendStore";

export function useCallEvents() {
  const setIncomingCall = useCallStore((s) => s.setIncomingCall);
  const updateCallState = useCallStore((s) => s.updateCallState);
  const endCall = useCallStore((s) => s.endCall);
  const updateDuration = useCallStore((s) => s.updateDuration);
  const activeCallStatus = useCallStore((s) => s.activeCall?.status);

  // Timer for updating call duration
  const durationTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Manage duration timer based on call status
  useEffect(() => {
    if (activeCallStatus === "in_progress" && !durationTimerRef.current) {
      // Start timer when call becomes active
      durationTimerRef.current = setInterval(() => {
        updateDuration();
      }, 1000);
    } else if (activeCallStatus !== "in_progress" && durationTimerRef.current) {
      // Stop timer when call ends or not in progress
      clearInterval(durationTimerRef.current);
      durationTimerRef.current = null;
    }

    return () => {
      if (durationTimerRef.current) {
        clearInterval(durationTimerRef.current);
        durationTimerRef.current = null;
      }
    };
  }, [activeCallStatus, updateDuration]);

  // Handle ToxAV events
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    onToxAvEvent((event: ToxAvEvent) => {
      switch (event.type) {
        case "IncomingCall": {
          // Get friend name from friend store
          const friends = useFriendStore.getState().friends;
          const friend = friends.find((f) => f.friend_number === event.data.friend_number);
          const friendName = friend?.name || `Friend ${event.data.friend_number}`;

          setIncomingCall({
            friendNumber: event.data.friend_number,
            friendName,
            audioEnabled: event.data.audio_enabled,
            videoEnabled: event.data.video_enabled,
            receivedAt: Date.now(),
          });
          break;
        }

        case "CallStateChange": {
          const status = event.data.state as CallStatus;
          updateCallState(event.data.friend_number, status, {
            sendingAudio: event.data.sending_audio,
            sendingVideo: event.data.sending_video,
            acceptingAudio: event.data.accepting_audio,
            acceptingVideo: event.data.accepting_video,
          });
          break;
        }

        case "CallEnded":
          endCall(event.data.friend_number, event.data.reason);
          break;

        case "AudioLevelUpdate":
          // TODO: Use for voice activity indicators
          break;
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [setIncomingCall, updateCallState, endCall]);
}
