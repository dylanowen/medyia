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
    status: TabStatus;
    isPlaying: boolean;
    displayName: string;
}

export interface BackendState {
    activeTab: TabKey,
    tabs: TabState[],
}

export async function createTab(source: MediaSource): Promise<TabKey> {
    console.log("[medyia] creating tab:", source);
    return await invoke("create_tab", {source});
}

export async function switchTab(key: TabKey) {
    console.log("[medyia] switching to tab:", key);
    await invoke("switch_tab", {key});
}

export async function closeTab(key: TabKey) {
    console.log("[medyia] closing tab:", key);
    await invoke("close_tab", {key});
}

export async function getBackendState(): Promise<BackendState> {
    console.log("[medyia] getting backend state");
    return await invoke("get_backend_state");
}

export async function getSources(): Promise<MediaDefinition[]> {
    return await invoke('get_sources')
}

