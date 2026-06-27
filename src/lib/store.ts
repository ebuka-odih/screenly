import { writable } from 'svelte/store';

export type AppState = 'idle' | 'recording' | 'export';

export const appState = writable<AppState>('idle');

export interface SourceOptions {
    screen: string;
    camera: string;
    cameraLabel: string;
    cameraType: 'none' | 'built-in' | 'continuity' | 'external' | 'unknown';
    mouseZoom: boolean;
    audio: {
        microphone: boolean;
        system: boolean;
    };
}

export const sourceOptions = writable<SourceOptions>({
    screen: 'Entire Screen',
    camera: 'default-user-camera',
    cameraLabel: 'Mac Front Camera',
    cameraType: 'built-in',
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

export interface CameraOption {
    deviceId: string;
    groupId?: string;
    label: string;
    kind: 'videoinput';
    cameraType: SourceOptions['cameraType'];
}

export const availableCameras = writable<CameraOption[]>([]);
export const availableMics = writable<MediaDeviceInfo[]>([]);
export const availableScreens = writable<ScreenSource[]>([]);

export const recordedVideoUrl = writable<string | null>(null);
export const recordedVideoMimeType = writable<string>('video/webm');
