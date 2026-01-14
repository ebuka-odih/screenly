<script lang="ts">
    import { sourceOptions, permissions } from '$lib/store';
</script>

<footer class="status-bar">
    <div class="status-items">
        <div class="status-group">
            <span class="status-dot green"></span>
            <span class="status-text">Permissions: OK</span>
        </div>
        <div class="divider"></div>
        <div class="status-group">
            <span class="status-label">Camera:</span>
            <span class="status-value">{$sourceOptions.camera}</span>
        </div>
        <div class="divider"></div>
        <div class="status-group">
            <span class="status-label">Audio:</span>
            <span class="status-value">{$sourceOptions.audio.microphone ? 'Mic' : 'None'}</span>
        </div>
    </div>
    <div class="version">
        v0.1.0
    </div>
</footer>

<style>
    .status-bar {
        height: 36px;
        background: rgba(0, 0, 0, 0.4);
        border-top: 1px solid rgba(255, 255, 255, 0.05);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 16px;
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.5);
    }

    .status-items {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    .status-group {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .status-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
    }

    .status-dot.green {
        background: #28cd41;
        box-shadow: 0 0 8px #28cd41;
    }

    .status-label {
        color: rgba(255, 255, 255, 0.3);
    }

    .status-value {
        color: rgba(255, 255, 255, 0.7);
    }

    .divider {
        width: 1px;
        height: 14px;
        background: rgba(255, 255, 255, 0.1);
    }

    .version {
        font-family: monospace;
        opacity: 0.4;
    }
</style>
