import {invoke} from "@tauri-apps/api/core";

export type MediaSource = string;

export interface MediaDefinition {
    id: MediaSource;
    name: string;
    iconUrl: string;
    defaultUrl: string;
    multiInstance: boolean;
}

export type TabKey = string;

export type TabStatus = "active" | "background" | "unloaded";

export interface TabState {
    key: string;
    source: string;
    url: string;
    isActive: boolean;
    isPlaying: boolean;
    displayName: string;
}

export interface AppState {
    media: Record<MediaSource, MediaState>,
}

export type MediaState = SingleMediaState | MultiMediaState;

export interface SingleMediaState {
    type: "single";
    source: MediaSource;
    tab: TabState | null;
}

export interface MultiMediaState {
    type: "multi";
    source: MediaSource;
    tabs: TabState[];
}

export async function createTab(source: MediaSource): Promise<TabKey> {
    console.debug("[medyia] creating tab:", source);
    return await invoke("create_tab", {source});
}

export async function switchSource(source: MediaSource) {
    console.debug("[medyia] switching to source:", source);
    await invoke("switch_source", {source});
}

export async function switchTab(key: TabKey) {
    console.debug("[medyia] switching to tab:", key);
    await invoke("switch_tab", {key});
}

export async function closeTab(key: TabKey) {
    console.debug("[medyia] closing tab:", key);
    await invoke("close_tab", {key});
}

export async function emitBackendState() {
    console.debug("[medyia] getting backend state");
    await invoke("emit_backend_state");
}

export async function getSources(): Promise<MediaDefinition[]> {
    return await invoke('get_sources')
}

