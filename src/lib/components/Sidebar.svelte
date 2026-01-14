<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import {
        appState,
        sourceOptions,
        availableCameras,
        availableScreens,
    } from "$lib/store";

    let selectedScreen = "Entire Screen";
    let selectedCamera = "None";
    let micEnabled = true;
    let systemAudioEnabled = false;
    let mouseZoomEnabled = false;
    let isCollapsed = false;

    let isBrowser = false;

    async function refreshDevices() {
        // Detect environment
        isBrowser =
            typeof window !== "undefined" &&
            !(window as any).__TAURI_INTERNALS__;

        // Request permission to unblock labels
        try {
            const stream = await navigator.mediaDevices.getUserMedia({
                video: true,
            });
            stream.getTracks().forEach((t) => t.stop());
        } catch (e) {
            console.warn("Camera permission prompt failed or denied:", e);
        }

        if (!isBrowser) {
            try {
                const cams = await invoke("list_cameras");
                const mappedCams = (cams as any[]).map((c) => ({
                    deviceId: c.id,
                    label: c.name,
                    kind: "videoinput" as const,
                }));
                availableCameras.set(mappedCams as MediaDeviceInfo[]);

                const screens = await invoke("list_screens");
                const mappedScreens = (screens as any[]).map((s) => ({
                    id: s.id,
                    name: s.name,
                    display_id: s.display_id,
                }));
                availableScreens.set(mappedScreens);

                if (
                    selectedScreen === "Entire Screen" &&
                    mappedScreens.length > 0
                ) {
                    selectedScreen = mappedScreens[0].id;
                }
                return;
            } catch (err) {
                console.error("Native bridge failed:", err);
            }
        }

        // Browser fallback (or if native bridge failed)
        const devices = await navigator.mediaDevices.enumerateDevices();
        const videoDevices = devices.filter((d) => d.kind === "videoinput");
        availableCameras.set(videoDevices);

        // In browser fallback, add a placeholder so the dropdown isn't empty
        availableScreens.set([
            {
                id: "browser-screen",
                name: "Browser Screen Share",
                display_id: 0,
            },
        ]);
        if (selectedScreen === "Entire Screen") {
            selectedScreen = "browser-screen";
        }
    }

    onMount(() => {
        refreshDevices();
        // Listen for device changes (Continuity Camera can be dynamic)
        navigator.mediaDevices.ondevicechange = refreshDevices;
    });

    function startRecording() {
        appState.set("recording");
    }

    function toggleSidebar() {
        isCollapsed = !isCollapsed;
    }

    $: sourceOptions.set({
        screen: selectedScreen,
        camera: selectedCamera,
        mouseZoom: mouseZoomEnabled,
        audio: {
            microphone: micEnabled,
            system: systemAudioEnabled,
        },
    });

    const screens = ["Entire Screen", "Window", "Region"];
</script>

<aside class="sidebar" class:collapsed={isCollapsed}>
    <button class="collapse-btn" on:click={toggleSidebar}>
        {isCollapsed ? "→" : "←"}
    </button>

    {#if !isCollapsed}
        <div class="sidebar-header">
            <div class="logo-group">
                <div class="logo-mark">S</div>
                <h1>Screenly</h1>
            </div>
            {#if isBrowser}
                <div class="browser-warning">
                    ⚠️ Browser Mode (Native Mac/iPhone features disabled).
                    <p>Run <code>npm run tauri dev</code> to unlock.</p>
                </div>
            {/if}
        </div>

        <div class="sidebar-content">
            <section class="section">
                <h2 class="section-title">Workflow</h2>
                <button class="btn-primary" on:click={startRecording}>
                    <span class="record-icon">●</span>
                    <span>Start Recording</span>
                </button>
            </section>

            <section class="section">
                <h2 class="section-title">Video Source</h2>
                <div class="input-group">
                    <label>Select Screen</label>
                    <select bind:value={selectedScreen} class="select">
                        {#each $availableScreens as s}
                            <option value={s.id}>{s.name}</option>
                        {/each}
                        <option value="window">Select Window...</option>
                        <option value="region">Select Region...</option>
                    </select>
                </div>

                <div class="input-group">
                    <label>Face Camera</label>
                    <div class="select-wrapper">
                        <select bind:value={selectedCamera} class="select">
                            <option value="None">None (Camera Disabled)</option>
                            {#each $availableCameras as cam}
                                <option value={cam.deviceId}
                                    >{cam.label || "Unknown Camera"}</option
                                >
                            {/each}
                        </select>
                        <button
                            class="refresh-mini"
                            on:click={refreshDevices}
                            title="Refresh Cameras">↻</button
                        >
                    </div>
                </div>
            </section>

            <section class="section">
                <h2 class="section-title">Audio Input</h2>
                <div class="checkbox-group">
                    <label class="custom-checkbox">
                        <input type="checkbox" bind:checked={micEnabled} />
                        <span class="checkmark"></span>
                        Microphone
                    </label>
                    <label class="custom-checkbox">
                        <input
                            type="checkbox"
                            bind:checked={systemAudioEnabled}
                        />
                        <span class="checkmark"></span>
                        System Audio
                    </label>
                    <label class="custom-checkbox">
                        <input
                            type="checkbox"
                            bind:checked={mouseZoomEnabled}
                        />
                        <span class="checkmark"></span>
                        Mouse Zoom (Focus Click)
                    </label>
                </div>
            </section>
        </div>

        <div class="sidebar-footer">
            <div class="storage-info">
                <div class="storage-bar">
                    <div class="storage-fill" style="width: 65%"></div>
                </div>
                <span>Disk Space: 12.4 GB free</span>
            </div>
        </div>
    {/if}
</aside>

<style>
    .sidebar {
        width: 280px;
        background: rgba(10, 10, 10, 0.8);
        backdrop-filter: blur(40px);
        border-right: 1px solid rgba(255, 255, 255, 0.08);
        display: flex;
        flex-direction: column;
        height: 100%;
        color: #fff;
        position: relative;
        transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        z-index: 10;
    }

    .sidebar.collapsed {
        width: 48px;
    }

    .collapse-btn {
        position: absolute;
        top: 24px;
        right: -12px;
        width: 24px;
        height: 24px;
        background: #fff;
        color: #000;
        border: none;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 0.8rem;
        z-index: 20;
        box-shadow: 0 4px 10px rgba(0, 0, 0, 0.3);
    }

    .sidebar-header {
        padding: 32px 24px;
    }

    .logo-group {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .logo-mark {
        width: 32px;
        height: 32px;
        background: linear-gradient(135deg, #ff4b2b, #ff416c);
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 900;
        font-size: 1.2rem;
    }

    .sidebar-header h1 {
        font-size: 1.1rem;
        font-weight: 800;
        margin: 0;
        letter-spacing: -0.02em;
    }

    .browser-warning {
        margin-top: 16px;
        background: rgba(255, 107, 0, 0.1);
        border: 1px solid rgba(255, 107, 0, 0.3);
        padding: 12px;
        border-radius: 8px;
        font-size: 0.75rem;
        color: #ffb347;
        line-height: 1.4;
    }

    .browser-warning code {
        background: rgba(0, 0, 0, 0.3);
        padding: 2px 4px;
        border-radius: 4px;
        color: #fff;
    }

    .sidebar-content {
        padding: 0 24px;
        display: flex;
        flex-direction: column;
        gap: 40px;
        flex: 1;
    }

    .section {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .section-title {
        font-size: 0.65rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: rgba(255, 255, 255, 0.3);
    }

    .btn-primary {
        background: #fff;
        color: #000;
        border: none;
        padding: 14px;
        border-radius: 12px;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 10px;
        font-size: 0.9rem;
    }

    .btn-primary:hover {
        transform: translateY(-2px);
        box-shadow: 0 10px 20px rgba(255, 255, 255, 0.1);
        background: #f0f0f0;
    }

    .record-icon {
        color: #ff3b30;
        animation: pulse 1s infinite;
        font-size: 0.8rem;
    }

    @keyframes pulse {
        0% {
            transform: scale(1);
            opacity: 1;
        }
        50% {
            transform: scale(1.2);
            opacity: 0.7;
        }
        100% {
            transform: scale(1);
            opacity: 1;
        }
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .input-group label {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.5);
    }

    .select-wrapper {
        display: flex;
        gap: 8px;
        align-items: center;
    }

    .select {
        flex: 1;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
        padding: 12px;
        border-radius: 10px;
        outline: none;
        font-size: 0.85rem;
        appearance: none;
    }

    .refresh-mini {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: rgba(255, 255, 255, 0.5);
        width: 38px;
        height: 38px;
        border-radius: 10px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
    }

    .refresh-mini:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
    }

    .checkbox-group {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    .custom-checkbox {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 0.85rem;
        color: rgba(255, 255, 255, 0.7);
        cursor: pointer;
    }

    .custom-checkbox input {
        display: none;
    }

    .checkmark {
        width: 18px;
        height: 18px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 4px;
        position: relative;
    }

    .custom-checkbox input:checked ~ .checkmark {
        background: #ff4b2b;
        border-color: #ff4b2b;
    }

    .custom-checkbox input:checked ~ .checkmark::after {
        content: "✓";
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        color: white;
        font-size: 10px;
    }

    .sidebar-footer {
        padding: 24px;
        border-top: 1px solid rgba(255, 255, 255, 0.05);
    }

    .storage-info {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .storage-bar {
        height: 4px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 2px;
        overflow: hidden;
    }

    .storage-fill {
        height: 100%;
        background: rgba(255, 255, 255, 0.3);
    }

    .storage-info span {
        font-size: 0.65rem;
        color: rgba(255, 255, 255, 0.3);
    }
</style>
