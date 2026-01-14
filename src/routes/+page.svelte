<script lang="ts">
    import Sidebar from '$lib/components/Sidebar.svelte';
    import MainContent from '$lib/components/MainContent.svelte';
    import StatusBar from '$lib/components/StatusBar.svelte';
    import { appState } from '$lib/store';
</script>

<div class="app-container" class:recording={$appState === 'recording'}>
    <div class="app-layout">
        {#if $appState !== 'recording'}
            <Sidebar />
        {/if}
        <div class="content-area">
            <MainContent />
            <StatusBar />
        </div>
    </div>
</div>

<style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');

    :global(body) {
        margin: 0;
        padding: 0;
        overflow: hidden;
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
        background: #000;
        color: #fff;
        height: 100vh;
        width: 100vw;
    }

    .app-container {
        height: 100vh;
        width: 100vw;
        display: flex;
        flex-direction: column;
        background: radial-gradient(circle at top left, #1a1a1a, #000);
    }

    .app-layout {
        flex: 1;
        display: flex;
        overflow: hidden;
    }

    .content-area {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    /* Transition for recording state */
    .app-container.recording .app-layout {
        transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
    }

    :global(*) {
        box-sizing: border-box;
    }

    :global(select), :global(button), :global(input) {
        font-family: inherit;
    }

    /* Custom Scrollbar */
    :global(::-webkit-scrollbar) {
        width: 8px;
    }
    :global(::-webkit-scrollbar-track) {
        background: transparent;
    }
    :global(::-webkit-scrollbar-thumb) {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
    }
    :global(::-webkit-scrollbar-thumb:hover) {
        background: rgba(255, 255, 255, 0.2);
    }
</style>
