import { writable } from 'svelte/store';

export type AppState = 'idle' | 'recording' | 'export';

export const appState = writable<AppState>('idle');

export interface SourceOptions {
    screen: string;
    camera: string;
    mouseZoom: boolean;
    audio: {
        microphone: boolean;
        system: boolean;
    };
}

export const sourceOptions = writable<SourceOptions>({
    screen: 'Entire Screen',
    camera: 'None',
    mouseZoom: false,
    audio: {
        microphone: true,
        system: false
    }
});

export const permissions = writable({
    screenRecording: 'loading', // 'loading' | 'granted' | 'denied'
    camera: 'loading',
    microphone: 'loading'
});

export const recordingTime = writable(0);

export interface ScreenSource {
    id: string;
    name: string;
    display_id: number;
}

export const availableCameras = writable<MediaDeviceInfo[]>([]);
export const availableMics = writable<MediaDeviceInfo[]>([]);
export const availableScreens = writable<ScreenSource[]>([]);

export const recordedVideoUrl = writable<string | null>(null);

