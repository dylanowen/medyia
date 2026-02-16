import {MediaSourceBar} from "./MediaSourceBar.tsx";
import {useCallback, useEffect, useState} from "react";
import * as commands from "../commands.ts";
import {BackendState, MediaSource, TabKey, TabState} from "../commands.ts";
import {Landing} from "./Landing.tsx";
import {useSources} from "../utils.tsx";
import {TabBar} from "./TabBar.tsx";
import {listen, UnlistenFn} from "@tauri-apps/api/event";

import './MediaSource.css'

export function MediaSourceController() {
    const sources = useSources();

    const [currentSource, setCurrentSource] = useState<MediaSource | null>(null)
    const [sourcesState, setSourcesState] = useState<Record<MediaSource, SourceState>>({});

    const refreshTabs = useCallback(({activeTab, tabs}: BackendState) => {
        const nextSourcesState: Record<MediaSource, SourceState> = {};
        for (const tab of tabs) {
            const sourceState: SourceState = nextSourcesState[tab.source] ?? {
                id: tab.source,
                tabs: [],
                activeTab: sourcesState[tab.source]?.activeTab ?? tab.key
            };

            sourceState.tabs.push(tab);

            if (tab.key == activeTab) {
                sourceState.activeTab = tab.key;
                setCurrentSource(tab.source);
            }

            nextSourcesState[tab.source] = sourceState;
        }

        setSourcesState(nextSourcesState);
    }, [sourcesState]);

    const changeSource = useCallback(async (source: MediaSource) => {
        const tabSourceState = sourcesState[source];
        if (!tabSourceState) {
            // create a tab for our new sources view
            await commands.createTab(source);
        } else {
            // tell the backend our current tab
            setCurrentSource(source);
            await commands.switchTab(tabSourceState.activeTab);
        }
    }, [sources, sourcesState]);


    useEffect(() => {
        let unlistenFn: UnlistenFn | null = null;
        (async () => {
            unlistenFn = await listen<BackendState>('BACKEND_STATE_EVENT', (event) => {
                refreshTabs(event.payload)
            });
            refreshTabs(await commands.getBackendState());
        })();

        //cleanup our listener
        return () => {
            unlistenFn && unlistenFn();
        };
    }, []);

    const currentSourceState = (currentSource != null) ? sourcesState[currentSource] : null

    return (<div className="media-source-controller">
        <MediaSourceBar
            currentSource={currentSource}
            changeSource={changeSource}
        />
        <div className="media-view">
            <MediaView
                sourceState={currentSourceState}
                changeSource={changeSource}
            />
        </div>
    </div>);
}

interface SourceState {
    id: MediaSource,
    tabs: TabState[],
    activeTab: TabKey,
}

interface MediaViewProps {
    sourceState: SourceState | null,
    changeSource: (source: MediaSource) => void
}

function MediaView({
                       sourceState,
                       changeSource,
                   }: MediaViewProps) {
    const sources = useSources();

    if (sourceState != null) {
        const {tabs, activeTab} = sourceState;
        if (sources.get(sourceState.id)!.multiInstance) {
            return <TabBar
                source={sourceState.id}
                tabs={tabs}
                activeTab={activeTab}
                onCreateTab={() => commands.createTab(sourceState.id)}
                onSwitchTab={commands.switchTab}
                onCloseTab={commands.closeTab}
            />;
        } else {
            return <></>;
        }
    } else {
        return <Landing onOpenService={changeSource}/>;
    }
}