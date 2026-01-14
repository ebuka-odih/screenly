<script lang="ts">
    import { appState, recordedVideoUrl, sourceOptions } from "$lib/store";
    import { get } from "svelte/store";

    let resolution = "1080p";
    let format = "MP4";

    function saveFile() {
        const url = get(recordedVideoUrl);
        if (url) {
            const a = document.createElement("a");
            a.href = url;
            a.download = `recording-${new Date().getTime()}.webm`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);

            // Success feedback and reset
            appState.set("idle");
            recordedVideoUrl.set(null);
        } else {
            alert("No recording found to save!");
        }
    }
</script>

<div class="export-panel glass">
    <div class="options-grid">
        <div class="option-item">
            <span class="label">Resolution</span>
            <select bind:value={resolution} class="export-select">
                <option value="1080p">1080p</option>
                <option value="4K">4K (Ultra HD)</option>
            </select>
        </div>

        <div class="option-item">
            <span class="label">Format</span>
            <select bind:value={format} class="export-select">
                <option value="MP4">MP4 (Recommended)</option>
                <option value="MOV">MOV (ProRes)</option>
            </select>
        </div>

        <div class="option-item full-width">
            <label class="toggle-container">
                <span class="label">Enable Click Zoom</span>
                <input
                    type="checkbox"
                    bind:checked={$sourceOptions.mouseZoom}
                />
                <span class="slider"></span>
            </label>
        </div>
    </div>

    <button class="btn-save" on:click={saveFile}> Save Recording </button>
</div>

<style>
    .export-panel {
        width: 100%;
        max-width: 600px;
        background: rgba(255, 255, 255, 0.05);
        backdrop-filter: blur(20px);
        padding: 30px;
        border-radius: 20px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        display: flex;
        flex-direction: column;
        gap: 24px;
    }

    .options-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 24px;
    }

    .option-item {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .full-width {
        grid-column: span 2;
    }

    .label {
        font-size: 0.7rem;
        font-weight: 600;
        text-transform: uppercase;
        color: rgba(255, 255, 255, 0.4);
    }

    .export-select {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
        padding: 12px;
        border-radius: 10px;
        outline: none;
    }

    .toggle-container {
        display: flex;
        justify-content: space-between;
        align-items: center;
        cursor: pointer;
    }

    .btn-save {
        background: #fff;
        color: #000;
        border: none;
        padding: 16px;
        border-radius: 12px;
        font-weight: 700;
        cursor: pointer;
        transition:
            transform 0.2s,
            background 0.2s;
    }

    .btn-save:hover {
        background: #f0f0f0;
        transform: translateY(-2px);
    }

    .btn-save:active {
        transform: translateY(0);
    }
</style>
