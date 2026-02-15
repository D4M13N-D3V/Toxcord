import {
  useRef,
  useEffect,
  useImperativeHandle,
  forwardRef,
  useCallback,
} from "react";

// Vertex shader - pass through position and texture coordinates
const VERTEX_SHADER = `
  attribute vec2 a_position;
  attribute vec2 a_texCoord;
  varying vec2 v_texCoord;
  void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
    v_texCoord = a_texCoord;
  }
`;

// Fragment shader - YUV to RGB conversion (BT.601)
const FRAGMENT_SHADER = `
  precision mediump float;
  uniform sampler2D u_yTexture;
  uniform sampler2D u_uTexture;
  uniform sampler2D u_vTexture;
  varying vec2 v_texCoord;

  void main() {
    float y = texture2D(u_yTexture, v_texCoord).r;
    float u = texture2D(u_uTexture, v_texCoord).r - 0.5;
    float v = texture2D(u_vTexture, v_texCoord).r - 0.5;

    // BT.601 YUV to RGB conversion
    float r = y + 1.402 * v;
    float g = y - 0.344 * u - 0.714 * v;
    float b = y + 1.772 * u;

    gl_FragColor = vec4(r, g, b, 1.0);
  }
`;

export interface VideoCanvasHandle {
  renderFrame: (
    y: Uint8Array,
    u: Uint8Array,
    v: Uint8Array,
    width: number,
    height: number,
  ) => void;
}

interface VideoCanvasProps {
  className?: string;
}

export const VideoCanvas = forwardRef<VideoCanvasHandle, VideoCanvasProps>(
  ({ className }, ref) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const glRef = useRef<WebGLRenderingContext | null>(null);
    const programRef = useRef<WebGLProgram | null>(null);
    const texturesRef = useRef<{
      y: WebGLTexture;
      u: WebGLTexture;
      v: WebGLTexture;
    } | null>(null);
    const lastDimensionsRef = useRef<{ width: number; height: number } | null>(
      null,
    );

    // Initialize WebGL
    useEffect(() => {
      const canvas = canvasRef.current;
      if (!canvas) return;

      const gl = canvas.getContext("webgl");
      if (!gl) {
        console.error("WebGL not supported");
        return;
      }

      glRef.current = gl;

      // Create shaders
      const vertexShader = createShader(gl, gl.VERTEX_SHADER, VERTEX_SHADER);
      const fragmentShader = createShader(
        gl,
        gl.FRAGMENT_SHADER,
        FRAGMENT_SHADER,
      );

      if (!vertexShader || !fragmentShader) {
        console.error("Failed to create shaders");
        return;
      }

      // Create program
      const program = createProgram(gl, vertexShader, fragmentShader);
      if (!program) {
        console.error("Failed to create WebGL program");
        return;
      }

      programRef.current = program;
      gl.useProgram(program);

      // Setup vertex buffer (full-screen quad)
      const positionBuffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
      // prettier-ignore
      const positions = new Float32Array([
        -1, -1,  0, 1,   // bottom-left
         1, -1,  1, 1,   // bottom-right
        -1,  1,  0, 0,   // top-left
         1,  1,  1, 0,   // top-right
      ]);
      gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

      // Setup attributes
      const positionLoc = gl.getAttribLocation(program, "a_position");
      const texCoordLoc = gl.getAttribLocation(program, "a_texCoord");

      gl.enableVertexAttribArray(positionLoc);
      gl.vertexAttribPointer(positionLoc, 2, gl.FLOAT, false, 16, 0);

      gl.enableVertexAttribArray(texCoordLoc);
      gl.vertexAttribPointer(texCoordLoc, 2, gl.FLOAT, false, 16, 8);

      // Create textures for Y, U, V planes
      const yTexture = createTexture(gl);
      const uTexture = createTexture(gl);
      const vTexture = createTexture(gl);

      if (!yTexture || !uTexture || !vTexture) {
        console.error("Failed to create textures");
        return;
      }

      texturesRef.current = { y: yTexture, u: uTexture, v: vTexture };

      // Set texture uniforms
      gl.uniform1i(gl.getUniformLocation(program, "u_yTexture"), 0);
      gl.uniform1i(gl.getUniformLocation(program, "u_uTexture"), 1);
      gl.uniform1i(gl.getUniformLocation(program, "u_vTexture"), 2);

      return () => {
        gl.deleteProgram(program);
        gl.deleteShader(vertexShader);
        gl.deleteShader(fragmentShader);
        if (texturesRef.current) {
          gl.deleteTexture(texturesRef.current.y);
          gl.deleteTexture(texturesRef.current.u);
          gl.deleteTexture(texturesRef.current.v);
        }
      };
    }, []);

    // Render a YUV frame
    const renderFrame = useCallback(
      (
        y: Uint8Array,
        u: Uint8Array,
        v: Uint8Array,
        width: number,
        height: number,
      ) => {
        const gl = glRef.current;
        const textures = texturesRef.current;
        const canvas = canvasRef.current;

        if (!gl || !textures || !canvas) return;

        // Update canvas size if dimensions changed
        const lastDims = lastDimensionsRef.current;
        if (!lastDims || lastDims.width !== width || lastDims.height !== height) {
          canvas.width = width;
          canvas.height = height;
          gl.viewport(0, 0, width, height);
          lastDimensionsRef.current = { width, height };
        }

        const uvWidth = width / 2;
        const uvHeight = height / 2;

        // Upload Y plane
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, textures.y);
        gl.texImage2D(
          gl.TEXTURE_2D,
          0,
          gl.LUMINANCE,
          width,
          height,
          0,
          gl.LUMINANCE,
          gl.UNSIGNED_BYTE,
          y,
        );

        // Upload U plane
        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(gl.TEXTURE_2D, textures.u);
        gl.texImage2D(
          gl.TEXTURE_2D,
          0,
          gl.LUMINANCE,
          uvWidth,
          uvHeight,
          0,
          gl.LUMINANCE,
          gl.UNSIGNED_BYTE,
          u,
        );

        // Upload V plane
        gl.activeTexture(gl.TEXTURE2);
        gl.bindTexture(gl.TEXTURE_2D, textures.v);
        gl.texImage2D(
          gl.TEXTURE_2D,
          0,
          gl.LUMINANCE,
          uvWidth,
          uvHeight,
          0,
          gl.LUMINANCE,
          gl.UNSIGNED_BYTE,
          v,
        );

        // Draw
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      },
      [],
    );

    // Expose renderFrame to parent
    useImperativeHandle(ref, () => ({ renderFrame }), [renderFrame]);

    return (
      <canvas
        ref={canvasRef}
        className={className}
        style={{ objectFit: "contain" }}
      />
    );
  },
);

VideoCanvas.displayName = "VideoCanvas";

// Helper functions
function createShader(
  gl: WebGLRenderingContext,
  type: number,
  source: string,
): WebGLShader | null {
  const shader = gl.createShader(type);
  if (!shader) return null;

  gl.shaderSource(shader, source);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.error("Shader compile error:", gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}

function createProgram(
  gl: WebGLRenderingContext,
  vertexShader: WebGLShader,
  fragmentShader: WebGLShader,
): WebGLProgram | null {
  const program = gl.createProgram();
  if (!program) return null;

  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error("Program link error:", gl.getProgramInfoLog(program));
    gl.deleteProgram(program);
    return null;
  }

  return program;
}

function createTexture(gl: WebGLRenderingContext): WebGLTexture | null {
  const texture = gl.createTexture();
  if (!texture) return null;

  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

  return texture;
}
