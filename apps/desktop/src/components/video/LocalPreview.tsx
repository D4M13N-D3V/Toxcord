import { useRef, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { VideoCanvas, VideoCanvasHandle } from "./VideoCanvas";
import { VideoFramePayload } from "../../api/calls";

interface LocalPreviewProps {
  className?: string;
}

export function LocalPreview({ className }: LocalPreviewProps) {
  const canvasRef = useRef<VideoCanvasHandle>(null);
  const [hasReceivedFrame, setHasReceivedFrame] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const frameCountRef = useRef(0);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    console.log("[LocalPreview] Setting up video frame listener for toxav://local-video");

    // Listen directly to see raw events
    listen<unknown>("toxav://local-video", (event) => {
      const payload = event.payload as Record<string, unknown>;

      // Check for error event first
      if (payload && payload.type === "VideoError" && payload.data) {
        const errorData = payload.data as { error: string };
        console.error("[LocalPreview] Video capture error:", errorData.error);
        setError(errorData.error);
        return;
      }

      // Log raw event for debugging
      frameCountRef.current++;
      if (frameCountRef.current <= 5) {
        console.log("[LocalPreview] Raw event received:", {
          eventName: event.event,
          payloadType: typeof event.payload,
          payloadKeys: payload && typeof payload === 'object' ? Object.keys(payload) : 'N/A',
        });
      }

      // Extract the frame data - handle both wrapped and unwrapped formats
      let frame: VideoFramePayload | null = null;

      if (payload && payload.type === "VideoFrame" && payload.data) {
        // Wrapped format: { type: "VideoFrame", data: VideoFramePayload }
        frame = payload.data as VideoFramePayload;
      } else if (payload && "width" in payload && "height" in payload && "data" in payload) {
        // Direct format: VideoFramePayload
        frame = payload as unknown as VideoFramePayload;
      }

      if (!frame) {
        console.warn("[LocalPreview] Could not extract frame from payload");
        return;
      }

      if (!canvasRef.current) {
        console.warn("[LocalPreview] Canvas ref not ready");
        return;
      }

      const { width, height, data } = frame;

      if (frameCountRef.current <= 3) {
        console.log(`[LocalPreview] Frame ${frameCountRef.current}: ${width}x${height}, data length: ${data?.length}`);
      }

      if (!data || data.length === 0) {
        console.warn("[LocalPreview] Empty frame data received");
        return;
      }

      const ySize = width * height;
      const uvSize = Math.floor(width / 2) * Math.floor(height / 2);
      const expectedSize = ySize + uvSize * 2;

      if (data.length !== expectedSize) {
        console.warn(`[LocalPreview] Unexpected data size: ${data.length}, expected: ${expectedSize}`);
      }

      // Convert number[] to Uint8Array and split into Y, U, V planes
      const yuvData = new Uint8Array(data);
      const y = yuvData.subarray(0, ySize);
      const u = yuvData.subarray(ySize, ySize + uvSize);
      const v = yuvData.subarray(ySize + uvSize, ySize + uvSize * 2);

      canvasRef.current.renderFrame(y, u, v, width, height);

      if (!hasReceivedFrame) {
        setHasReceivedFrame(true);
      }
    }).then((fn) => {
      unlisten = fn;
      console.log("[LocalPreview] Video frame listener registered successfully");
    }).catch((err) => {
      console.error("[LocalPreview] Failed to register listener:", err);
    });

    return () => {
      console.log("[LocalPreview] Cleaning up video frame listener");
      unlisten?.();
    };
  }, [hasReceivedFrame]);

  return (
    <div className={`relative overflow-hidden rounded-lg bg-black ${className || ""}`}>
      <VideoCanvas ref={canvasRef} className="h-full w-full" />
      {error && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-[#2b2d31] p-2">
          <svg className="h-6 w-6 text-discord-red mb-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
          <span className="text-[10px] text-discord-muted text-center">No camera</span>
        </div>
      )}
      {!hasReceivedFrame && !error && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
        </div>
      )}
    </div>
  );
}
