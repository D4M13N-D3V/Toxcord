import { useRef, useEffect, useState } from "react";
import { VideoCanvas, VideoCanvasHandle } from "./VideoCanvas";
import { onRemoteVideoFrame, VideoFramePayload } from "../../api/calls";

interface RemoteVideoProps {
  friendNumber: number;
  className?: string;
}

export function RemoteVideo({ friendNumber, className }: RemoteVideoProps) {
  const canvasRef = useRef<VideoCanvasHandle>(null);
  const [hasReceivedFrame, setHasReceivedFrame] = useState(false);
  const frameCountRef = useRef(0);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const handleFrame = (frame: VideoFramePayload) => {
      // Only render frames from the specified friend
      if (frame.friend_number !== friendNumber) return;

      if (!canvasRef.current) {
        console.warn("[RemoteVideo] Canvas ref not ready");
        return;
      }

      const { width, height, data } = frame;

      // Debug logging for first few frames
      frameCountRef.current++;
      if (frameCountRef.current <= 3) {
        console.log(`[RemoteVideo] Frame ${frameCountRef.current} from friend ${friendNumber}: ${width}x${height}, data length: ${data?.length}`);
      }

      if (!data || data.length === 0) {
        console.warn("[RemoteVideo] Empty frame data received");
        return;
      }

      const ySize = width * height;
      const uvSize = Math.floor(width / 2) * Math.floor(height / 2);
      const expectedSize = ySize + uvSize * 2;

      if (data.length !== expectedSize) {
        console.warn(`[RemoteVideo] Unexpected data size: ${data.length}, expected: ${expectedSize}`);
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
    };

    console.log(`[RemoteVideo] Setting up video frame listener for friend ${friendNumber}`);
    onRemoteVideoFrame(handleFrame).then((fn) => {
      unlisten = fn;
      console.log(`[RemoteVideo] Video frame listener ready for friend ${friendNumber}`);
    });

    return () => {
      console.log(`[RemoteVideo] Cleaning up video frame listener for friend ${friendNumber}`);
      unlisten?.();
    };
  }, [friendNumber, hasReceivedFrame]);

  return (
    <div className={`relative overflow-hidden rounded-lg bg-black ${className || ""}`}>
      <VideoCanvas ref={canvasRef} className="h-full w-full" />
      {!hasReceivedFrame && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent" />
        </div>
      )}
    </div>
  );
}
