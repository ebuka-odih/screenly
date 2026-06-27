<script lang="ts">
    import { onMount, onDestroy, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import {
        appState,
        recordingTime,
        sourceOptions,
        recordedVideoUrl,
        recordedVideoMimeType,
    } from "$lib/store";
    import ExportPanel from "./ExportPanel.svelte";

    const RECORDING_FRAME_RATE = 30;

    let clickHistory: { x: number; y: number; time: number }[] = [];
    let recordingStartTimestamp = 0;
    let activeZoom: { x: number; y: number; scale: number } | null = null;
    let isTauri =
        typeof window !== "undefined" && !!(window as any).__TAURI_INTERNALS__;

    let timerId: any;
    let videoElement: HTMLVideoElement;
    let screenVideoElement: HTMLVideoElement;
    let previewVideoElement: HTMLVideoElement;
    let canvasElement: HTMLCanvasElement;
    let canvasCtx: CanvasRenderingContext2D | null = null;
    let animationId: number;
    let stream: MediaStream | null = null;
    let screenStream: MediaStream | null = null;
    let audioStream: MediaStream | null = null;
    let mediaRecorder: MediaRecorder | null = null;
    let recordedChunks: Blob[] = [];
    let audioContext: AudioContext | null = null;
    let captureSessionActive = false;
    let isStoppingRecording = false;
    let recordingError = "";

    function getSupportedMimeType() {
        const supportedTypes = [
            'video/mp4; codecs="avc1.42E01E, mp4a.40.2"',
            "video/webm; codecs=vp9,opus",
            "video/webm; codecs=vp8,opus",
            "video/webm",
        ];

        return (
            supportedTypes.find((type) => MediaRecorder.isTypeSupported(type)) ||
            ""
        );
    }

    function getSelectedCameraConstraintCandidates(): MediaStreamConstraints[] {
        const isIphoneCamera = $sourceOptions.cameraType === "continuity";
        const qualityConstraints = {
            width: { ideal: isIphoneCamera ? 1920 : 1280 },
            height: { ideal: isIphoneCamera ? 1080 : 720 },
            frameRate: {
                ideal: RECORDING_FRAME_RATE,
                max: RECORDING_FRAME_RATE,
            },
        };

        if ($sourceOptions.camera === "default-user-camera") {
            return [
                {
                    video: true,
                    audio: false,
                },
                {
                    video: {
                        ...qualityConstraints,
                        facingMode: { ideal: "user" },
                    },
                    audio: false,
                },
            ];
        }

        return [
            {
                video: {
                    ...qualityConstraints,
                    deviceId: { exact: $sourceOptions.camera },
                },
                audio: false,
            },
            {
                video: {
                    ...qualityConstraints,
                    deviceId: { ideal: $sourceOptions.camera },
                },
                audio: false,
            },
            {
                video: true,
                audio: false,
            },
        ];
    }

    function describeMediaError(err: unknown) {
        if (err instanceof DOMException) {
            if (err.name === "NotAllowedError") {
                return "Camera permission is blocked. Allow camera access for Screenly, then try recording again.";
            }
            if (err.name === "NotFoundError") {
                return "No camera was found by macOS/WebKit. Check that the Mac camera is available and not in use by another app.";
            }
            if (
                err.name === "OverconstrainedError" ||
                err.name === "ConstraintNotSatisfiedError"
            ) {
                return "The selected camera rejected the requested settings. Trying the default camera failed too.";
            }
            if (err.name === "NotReadableError") {
                return "The camera is busy or unavailable. Close other apps using the camera and try again.";
            }

            return `${err.name}: ${err.message}`;
        }

        return err instanceof Error ? err.message : String(err);
    }

    async function openSelectedCamera() {
        const constraints = getSelectedCameraConstraintCandidates();
        let lastError: unknown = null;

        for (const constraint of constraints) {
            try {
                return await navigator.mediaDevices.getUserMedia(constraint);
            } catch (err) {
                lastError = err;
                console.warn("Camera constraint failed:", constraint, err);
            }
        }

        throw new Error(describeMediaError(lastError));
    }

    async function attachStreamToVideo(
        element: HTMLVideoElement | undefined,
        mediaStream: MediaStream,
    ) {
        if (!element) return;

        element.srcObject = mediaStream;
        await element.play().catch((err) => {
            console.warn("Video preview could not autoplay:", err);
        });
    }

    function startRecorder(compositeStream: MediaStream) {
        const mimeType = getSupportedMimeType();

        mediaRecorder = mimeType
            ? new MediaRecorder(compositeStream, { mimeType })
            : new MediaRecorder(compositeStream);

        mediaRecorder.ondataavailable = (e) => {
            if (e.data && e.data.size > 0) recordedChunks.push(e.data);
        };

        mediaRecorder.onstop = () => {
            const finalBlob = new Blob(recordedChunks, {
                type: mediaRecorder?.mimeType || "video/mp4",
            });
            const url = URL.createObjectURL(finalBlob);
            recordedVideoUrl.set(url);
            recordedVideoMimeType.set(finalBlob.type || "video/webm");
            recordedChunks = [];
            stopAllStreams();
            captureSessionActive = false;
            isStoppingRecording = false;
            appState.set("export");
        };

        mediaRecorder.start(1000);
        recordingStartTimestamp = Date.now();
        startRecordingTimer();

        if (isTauri) {
            invoke("start_recording").catch((err) =>
                console.warn("Failed to start native recording state:", err),
            );
        }
    }

    async function startCameraOnlyCapture(message = "") {
        if (!stream) {
            throw new Error(
                recordingError ||
                    "Camera could not be opened. Check camera permission and try again.",
            );
        }

        const cameraTrack = stream.getVideoTracks()[0];
        const settings = cameraTrack.getSettings();
        const width = settings.width || videoElement?.videoWidth || 1280;
        const height = settings.height || videoElement?.videoHeight || 720;

        canvasElement = document.createElement("canvas");
        canvasElement.width = width;
        canvasElement.height = height;
        canvasCtx = canvasElement.getContext("2d");

        const compositeStream =
            canvasElement.captureStream(RECORDING_FRAME_RATE);

        if (audioStream && audioStream.getAudioTracks().length > 0) {
            audioStream
                .getAudioTracks()
                .forEach((track) => compositeStream.addTrack(track));
        }

        const draw = () => {
            if (!canvasCtx) return;

            canvasCtx.fillStyle = "#000";
            canvasCtx.fillRect(0, 0, width, height);

            if (videoElement && videoElement.readyState >= 2) {
                canvasCtx.save();
                canvasCtx.translate(width, 0);
                canvasCtx.scale(-1, 1);
                canvasCtx.drawImage(videoElement, 0, 0, width, height);
                canvasCtx.restore();
            }

            animationId = requestAnimationFrame(draw);
        };
        draw();

        if (message) recordingError = message;
        startRecorder(compositeStream);
    }

    async function startCapture() {
        await tick();
        recordedChunks = [];
        clickHistory = [];
        isStoppingRecording = false;
        recordingError = "";
        recordedVideoUrl.set(null);

        if (!$sourceOptions.mouseZoom) {
            // Optional: notify Rust to NOT track if we want to be even more efficient
        }

        // 1. Start Camera if enabled
        if ($sourceOptions.camera !== "None") {
            try {
                stream = await openSelectedCamera();
                await attachStreamToVideo(videoElement, stream);
            } catch (err) {
                console.error("Error accessing camera:", err);
                recordingError = describeMediaError(err);
            }
        }

        // 2. Start Microphone if enabled
        if ($sourceOptions.audio.microphone) {
            try {
                audioStream = await navigator.mediaDevices.getUserMedia({
                    audio: true,
                });
            } catch (err) {
                console.error("Error accessing microphone:", err);
            }
        }

        if ($sourceOptions.screen === "camera-only") {
            try {
                await startCameraOnlyCapture();
            } catch (err) {
                console.error("Error starting camera-only recording:", err);
                recordingError =
                    err instanceof Error
                        ? err.message
                        : "Camera recording could not start.";
                captureSessionActive = false;
                isStoppingRecording = false;
                stopAllStreams();
                clearInterval(timerId);
                appState.set("idle");
            }
            return;
        }

        // 3. Start Screen Capture
        try {
            const displayVideoConstraints = {
                cursor: "always",
                frameRate: {
                    ideal: RECORDING_FRAME_RATE,
                    max: RECORDING_FRAME_RATE,
                },
            } as MediaTrackConstraints & { cursor: string };

            screenStream = await navigator.mediaDevices.getDisplayMedia({
                video: displayVideoConstraints,
                audio: $sourceOptions.audio.system,
            });
            await attachStreamToVideo(screenVideoElement, screenStream);

            // Setup Canvas Compositor
            const track = screenStream.getVideoTracks()[0];
            const settings = track.getSettings();
            const width = settings.width || 1920;
            const height = settings.height || 1080;

            canvasElement = document.createElement("canvas");
            canvasElement.width = width;
            canvasElement.height = height;
            canvasCtx = canvasElement.getContext("2d");

            const compositeStream =
                canvasElement.captureStream(RECORDING_FRAME_RATE);

            // --- Audio Mixing Logic ---
            audioContext = new AudioContext();
            const dest = audioContext.createMediaStreamDestination();
            let hasAudio = false;

            // Mix Microphone
            if (audioStream && audioStream.getAudioTracks().length > 0) {
                const micSource =
                    audioContext.createMediaStreamSource(audioStream);
                micSource.connect(dest);
                hasAudio = true;
            }

            // Mix System Audio (from screen share)
            if (screenStream.getAudioTracks().length > 0) {
                const systemSource =
                    audioContext.createMediaStreamSource(screenStream);
                systemSource.connect(dest);
                hasAudio = true;
            }

            // Add the mixed track to our composite stream
            if (hasAudio) {
                dest.stream
                    .getAudioTracks()
                    .forEach((track) => compositeStream.addTrack(track));
            }
            // ---------------------------

            const draw = () => {
                if (!canvasCtx || !screenStream) return;

                // 1. Clear Canvas
                canvasCtx.fillStyle = "#000";
                canvasCtx.fillRect(0, 0, width, height);

                // 2. Draw Screen
                if (screenVideoElement && screenVideoElement.readyState >= 2) {
                    canvasCtx.drawImage(
                        screenVideoElement,
                        0,
                        0,
                        width,
                        height,
                    );
                }

                // 3. Draw Camera Overlay (bottom right)
                if (stream && videoElement && videoElement.readyState >= 2) {
                    const videoTrack = stream.getVideoTracks()[0];
                    const settings = videoTrack.getSettings();
                    const isPortrait =
                        settings.height && settings.width
                            ? settings.height > settings.width
                            : false;

                    const camW = width * (isPortrait ? 0.15 : 0.2);
                    const camH = isPortrait ? (camW * 16) / 9 : (camW * 9) / 16;

                    const padding = 60;
                    const x = width - camW - padding;
                    const y = height - camH - padding;

                    // Draw Shadow for Cam Card
                    canvasCtx.save();
                    canvasCtx.shadowBlur = 30;
                    canvasCtx.shadowColor = "rgba(0,0,0,0.5)";
                    canvasCtx.fillStyle = "rgba(255,255,255,0.1)";
                    canvasCtx.beginPath();
                    canvasCtx.roundRect(x, y, camW, camH, 24);
                    canvasCtx.fill();
                    canvasCtx.restore();

                    // Mask and Draw Camera
                    canvasCtx.save();
                    canvasCtx.beginPath();
                    canvasCtx.roundRect(x, y, camW, camH, 24);
                    canvasCtx.clip();

                    // Draw Mirrored Camera
                    // We also need to handle the cropping if the aspect ratios don't match exactly
                    canvasCtx.translate(x + camW, y);
                    canvasCtx.scale(-1, 1);

                    const streamW = settings.width || videoElement.videoWidth;
                    const streamH = settings.height || videoElement.videoHeight;
                    const streamAspect = streamW / streamH;
                    const targetAspect = camW / camH;

                    let drawW = camW;
                    let drawH = camH;
                    let offsetX = 0;
                    let offsetY = 0;

                    if (streamAspect > targetAspect) {
                        drawW = camH * streamAspect;
                        offsetX = -(drawW - camW) / 2;
                    } else {
                        drawH = camW / streamAspect;
                        offsetY = -(drawH - camH) / 2;
                    }

                    canvasCtx.drawImage(
                        videoElement,
                        offsetX,
                        offsetY,
                        drawW,
                        drawH,
                    );
                    canvasCtx.restore();

                    // Draw Glass Border
                    canvasCtx.strokeStyle = "rgba(255,255,255,0.2)";
                    canvasCtx.lineWidth = 2;
                    canvasCtx.beginPath();
                    canvasCtx.roundRect(x, y, camW, camH, 24);
                    canvasCtx.stroke();
                }

                animationId = requestAnimationFrame(draw);
            };
            draw();

            startRecorder(compositeStream);

            track.onended = () => stopRecording();
        } catch (err) {
            console.error("Error accessing screen:", err);
            if (stream) {
                try {
                    await startCameraOnlyCapture(
                        "Screen capture failed. Recording camera only.",
                    );
                    return;
                } catch (cameraOnlyErr) {
                    console.error(
                        "Camera-only fallback failed:",
                        cameraOnlyErr,
                    );
                }
            }
            recordingError = "Screen capture was cancelled or unavailable.";
            captureSessionActive = false;
            isStoppingRecording = false;
            if (isTauri) {
                invoke("stop_recording").catch((invokeErr) =>
                    console.warn("Failed to stop native recording:", invokeErr),
                );
            }
            stopAllStreams();
            clearInterval(timerId);
            appState.set("idle");
        }
    }

    function startRecordingTimer() {
        clearInterval(timerId);
        recordingTime.set(0);
        timerId = setInterval(() => {
            recordingTime.update((n) => n + 1);
        }, 1000);
    }

    function stopAllStreams() {
        cancelAnimationFrame(animationId);
        if (stream) {
            stream.getTracks().forEach((track) => track.stop());
            stream = null;
        }
        if (screenStream) {
            screenStream.getTracks().forEach((track) => track.stop());
            screenStream = null;
        }
        if (audioStream) {
            audioStream.getTracks().forEach((track) => track.stop());
            audioStream = null;
        }
        if (audioContext && audioContext.state !== "closed") {
            audioContext.close();
        }
        audioContext = null;
    }

    function stopRecording() {
        if (isStoppingRecording) return;

        isStoppingRecording = true;
        if (isTauri) {
            invoke("stop_recording").catch((err) =>
                console.warn("Failed to stop native recording:", err),
            );
        }
        if (mediaRecorder && mediaRecorder.state !== "inactive") {
            mediaRecorder.requestData();
            mediaRecorder.stop();
        } else {
            stopAllStreams();
            captureSessionActive = false;
            isStoppingRecording = false;
            appState.set("idle");
        }
        clearInterval(timerId);
    }

    // Effect logic for playback zoom
    let playbackTime = 0;
    function handleTimeUpdate(e: any) {
        playbackTime = e.target.currentTime * 1000; // in ms
        if ($sourceOptions.mouseZoom) {
            const currentClick = clickHistory.find(
                (c) => playbackTime >= c.time && playbackTime <= c.time + 1000,
            );
            if (currentClick) {
                // Calculate smooth zoom
                const progress = (playbackTime - currentClick.time) / 1000;
                const scale = 1 + Math.sin(progress * Math.PI) * 0.5; // Zoom in and out
                activeZoom = { x: currentClick.x, y: currentClick.y, scale };
            } else {
                activeZoom = null;
            }
        } else {
            activeZoom = null;
        }
    }

    let unlistenFn: (() => void) | null = null;
    onMount(() => {
        let destroyed = false;

        if (isTauri) {
            listen("mouse-click", (event: any) => {
                    if ($appState === "recording") {
                        const click = event.payload;
                        clickHistory.push({
                            x: click.x,
                            y: click.y,
                            time: Date.now() - recordingStartTimestamp,
                        });
                    }
                })
                .then((unlisten) => {
                    if (destroyed) {
                        unlisten();
                        return;
                    }
                    unlistenFn = unlisten;
                })
                .catch((err) => {
                    console.error("Failed to setup mouse tracking:", err);
                });
        }

        return () => {
            destroyed = true;
            if (unlistenFn) unlistenFn();
        };
    });

    $: if ($appState === "recording" && !captureSessionActive) {
        captureSessionActive = true;
        recordingTime.set(0);
        startCapture();
    } else if ($appState === "idle") {
        stopAllStreams();
        captureSessionActive = false;
        isStoppingRecording = false;
        clearInterval(timerId);
    } else {
        clearInterval(timerId);
    }

    onDestroy(() => {
        clearInterval(timerId);
        stopAllStreams();
    });

    function formatTime(seconds: number) {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }

    // Draggable Cam logic
    let camPos = { x: 0, y: 0 };
    let isDragging = false;
    let startPos = { x: 0, y: 0 };

    function handleMouseDown(e: MouseEvent) {
        isDragging = true;
        startPos = { x: e.clientX - camPos.x, y: e.clientY - camPos.y };
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isDragging) return;
        camPos = {
            x: e.clientX - startPos.x,
            y: e.clientY - startPos.y,
        };
    }

    function handleMouseUp() {
        isDragging = false;
    }

    function handleOverlayKeydown(e: KeyboardEvent) {
        const step = e.shiftKey ? 30 : 10;

        if (e.key === "ArrowUp") {
            camPos = { ...camPos, y: camPos.y - step };
        } else if (e.key === "ArrowDown") {
            camPos = { ...camPos, y: camPos.y + step };
        } else if (e.key === "ArrowLeft") {
            camPos = { ...camPos, x: camPos.x - step };
        } else if (e.key === "ArrowRight") {
            camPos = { ...camPos, x: camPos.x + step };
        } else {
            return;
        }

        e.preventDefault();
    }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<main class="main-content">
    {#if $appState === "idle"}
        <div class="view-container idle-view">
            <div class="preview-stage">
                <div class="preview-card glass">
                    <div class="shimmer"></div>
                    <div class="preview-content">
                        <div class="icon-circle">
                            <span class="preview-icon">📽️</span>
                        </div>
                        <h3>Ready to capture?</h3>
                        <p>
                            Select your sources in the sidebar and hit Record.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    {:else if $appState === "recording"}
        <div class="view-container recording-view">
            <div class="live-canvas">
                <div class="canvas-grid"></div>

                <!-- Screen Feed -->
                <video
                    bind:this={screenVideoElement}
                    autoplay
                    playsinline
                    muted
                    class="screen-video"
                >
                    <track kind="captions" />
                </video>

                <!-- Draggable Face Cam -->
                <div
                    class="face-cam-overlay"
                    style="transform: translate({camPos.x}px, {camPos.y}px)"
                    on:mousedown={handleMouseDown}
                    on:keydown={handleOverlayKeydown}
                    role="button"
                    tabindex="0"
                    aria-label="Camera overlay"
                >
                    <div class="cam-indicator">LIVE</div>
                    <video
                        bind:this={videoElement}
                        autoplay
                        playsinline
                        muted
                        class="cam-video"
                    >
                        <track kind="captions" />
                    </video>
                    {#if !stream}
                        <div class="cam-lens"></div>
                        <span class="cam-label"
                            >{$sourceOptions.camera === "None"
                                ? "Camera Off"
                                : "Camera Unavailable"}</span
                        >
                    {/if}
                </div>
            </div>

            <div class="floating-controls">
                <div class="rec-badge glass">
                    <span class="rec-dot"></span>
                    <span class="timer-text">{formatTime($recordingTime)}</span>
                </div>
                <button class="stop-trigger" on:click={stopRecording}>
                    <div class="stop-icon"></div>
                    <span>End Session</span>
                </button>
            </div>
            {#if recordingError}
                <div class="recording-warning glass">{recordingError}</div>
            {/if}
        </div>
    {:else if $appState === "export"}
        <div class="view-container export-view">
            <div class="export-layout">
                <div class="video-preview-card glass">
                    {#if $recordedVideoUrl}
                        <div
                            class="player-wrapper"
                            style="overflow: hidden; border-radius: 12px; position: relative;"
                        >
                            <video
                                bind:this={previewVideoElement}
                                src={$recordedVideoUrl}
                                controls
                                class="preview-player"
                                on:timeupdate={handleTimeUpdate}
                                style={activeZoom
                                    ? `transform-origin: ${activeZoom.x}px ${activeZoom.y}px; transform: scale(${activeZoom.scale}); transition: transform 0.1s ease-out;`
                                    : "transition: transform 0.3s ease-out;"}
                            >
                                <track kind="captions" />
                            </video>
                            {#if activeZoom}
                                <div
                                    class="zoom-indicator"
                                    style="position: absolute; left: ${activeZoom.x}px; top: ${activeZoom.y}px; width: 40px; height: 40px; border: 2px solid white; border-radius: 50%; transform: translate(-50%, -50%); pointer-events: none; mix-blend-mode: difference;"
                                ></div>
                            {/if}
                        </div>
                    {:else}
                        <div class="video-mock">
                            <div class="loading-spinner"></div>
                            <span>Processing recording...</span>
                        </div>
                    {/if}
                </div>
                <div class="export-controls-container">
                    <h2 class="export-title">Export Settings</h2>
                    <ExportPanel />
                </div>
            </div>
        </div>
    {/if}
</main>

<style>
    .main-content {
        flex: 1;
        background: #000;
        position: relative;
        overflow: hidden;
    }

    .view-container {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        animation: fadeIn 0.4s ease-out;
    }

    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    /* Idle View Styling */
    .idle-view {
        align-items: center;
        justify-content: center;
        background: radial-gradient(circle at center, #111 0%, #000 100%);
    }

    .preview-stage {
        perspective: 1000px;
    }

    .preview-card {
        width: 480px;
        height: 280px;
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        text-align: center;
        border-radius: 24px;
        overflow: hidden;
        border: 1px solid rgba(255, 255, 255, 0.1);
        box-shadow: 0 30px 60px rgba(0, 0, 0, 0.8);
        transform: rotateX(5deg);
    }

    .preview-content {
        z-index: 2;
        padding: 40px;
    }

    .icon-circle {
        width: 64px;
        height: 64px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        margin: 0 auto 20px;
        font-size: 2rem;
    }

    .preview-card h3 {
        margin: 0 0 10px;
        font-size: 1.5rem;
        color: #fff;
    }

    .preview-card p {
        margin: 0;
        color: rgba(255, 255, 255, 0.4);
        font-size: 0.95rem;
        line-height: 1.5;
    }

    .shimmer {
        position: absolute;
        top: 0;
        left: -100%;
        width: 200%;
        height: 100%;
        background: linear-gradient(
            90deg,
            transparent,
            rgba(255, 255, 255, 0.05),
            transparent
        );
        animation: shimmer 5s infinite;
    }

    @keyframes shimmer {
        0% {
            left: -100%;
        }
        100% {
            left: 100%;
        }
    }

    /* Recording View Styling */
    .recording-view {
        padding: 20px;
        background: #050505;
    }

    .live-canvas {
        flex: 1;
        background: #0d0d0d;
        border-radius: 20px;
        border: 1px solid rgba(255, 255, 255, 0.05);
        position: relative;
        overflow: hidden;
        box-shadow: inset 0 0 100px rgba(0, 0, 0, 1);
    }

    .canvas-grid {
        position: absolute;
        inset: 0;
        background-image: linear-gradient(
                rgba(255, 255, 255, 0.02) 1px,
                transparent 1px
            ),
            linear-gradient(
                90deg,
                rgba(255, 255, 255, 0.02) 1px,
                transparent 1px
            );
        background-size: 40px 40px;
    }

    .face-cam-overlay {
        position: absolute;
        bottom: 40px;
        right: 40px;
        width: 240px;
        aspect-ratio: 16/9;
        background: #1a1a1a;
        border-radius: 16px;
        border: 2px solid rgba(255, 255, 255, 0.2);
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
        cursor: grab;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        z-index: 100;
    }

    .screen-video {
        width: 100%;
        height: 100%;
        object-fit: contain;
        position: absolute;
        inset: 0;
    }

    .cam-video {
        width: 100%;
        height: 100%;
        object-fit: cover;
        transform: scaleX(-1); /* Mirror effect */
    }

    .face-cam-overlay:active {
        cursor: grabbing;
    }

    .cam-indicator {
        position: absolute;
        top: 12px;
        left: 12px;
        background: rgba(255, 59, 48, 0.8);
        color: white;
        font-size: 10px;
        font-weight: 800;
        padding: 2px 6px;
        border-radius: 4px;
        letter-spacing: 0.1em;
    }

    .cam-lens {
        width: 40px;
        height: 40px;
        background: #000;
        border-radius: 50%;
        border: 4px solid #222;
        margin-bottom: 12px;
    }

    .cam-label {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.3);
        text-transform: uppercase;
        font-weight: 600;
    }

    .floating-controls {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 20px;
        padding-top: 20px;
    }

    .rec-badge {
        background: rgba(255, 255, 255, 0.05);
        padding: 10px 20px;
        border-radius: 100px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        display: flex;
        align-items: center;
        gap: 12px;
        min-width: 120px;
        justify-content: center;
    }

    .rec-dot {
        width: 10px;
        height: 10px;
        background: #ff3b30;
        border-radius: 50%;
        box-shadow: 0 0 10px #ff3b30;
        animation: blink 1s step-end infinite;
    }

    @keyframes blink {
        50% {
            opacity: 0;
        }
    }

    .timer-text {
        font-family: "JetBrains Mono", monospace;
        font-weight: 700;
        font-size: 1rem;
        color: #fff;
    }

    .recording-warning {
        position: absolute;
        left: 50%;
        bottom: 92px;
        transform: translateX(-50%);
        border: 1px solid rgba(255, 184, 77, 0.2);
        border-radius: 8px;
        color: #ffcc80;
        font-size: 0.8rem;
        padding: 10px 14px;
        max-width: min(520px, calc(100vw - 48px));
        text-align: center;
    }

    .stop-trigger {
        background: #ff3b30;
        color: white;
        border: none;
        padding: 12px 24px;
        border-radius: 100px;
        display: flex;
        align-items: center;
        gap: 12px;
        font-weight: 700;
        cursor: pointer;
        transition:
            transform 0.2s,
            box-shadow 0.2s;
        box-shadow: 0 10px 20px rgba(255, 59, 48, 0.3);
    }

    .stop-trigger:hover {
        transform: translateY(-2px);
        box-shadow: 0 15px 30px rgba(255, 59, 48, 0.4);
    }

    .stop-icon {
        width: 14px;
        height: 14px;
        background: white;
        border-radius: 3px;
    }

    /* Export View Styling */
    .export-view {
        padding: 40px;
        overflow-y: auto;
    }

    .export-layout {
        max-width: 1000px;
        margin: 0 auto;
        display: grid;
        grid-template-columns: 1.5fr 1fr;
        gap: 40px;
    }

    .video-preview-card {
        background: rgba(255, 255, 255, 0.03);
        border-radius: 24px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        padding: 20px;
        height: fit-content;
    }

    .video-mock {
        width: 100%;
        aspect-ratio: 16/9;
        background: #000;
        border-radius: 12px;
        position: relative;
        overflow: hidden;
    }

    .preview-player {
        width: 100%;
        max-height: 80vh;
        border-radius: 12px;
        background: #000;
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
    }

    .loading-spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(255, 255, 255, 0.1);
        border-top-color: #fff;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-bottom: 20px;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .export-title {
        font-size: 1.5rem;
        margin: 0 0 24px;
        font-weight: 700;
    }

    .glass {
        background: rgba(255, 255, 255, 0.03);
        backdrop-filter: blur(20px);
    }
</style>
