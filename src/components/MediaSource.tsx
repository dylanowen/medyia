import {MediaSourceBar} from "./MediaSourceBar.tsx";
import {useCallback, useEffect, useState} from "react";
import * as commands from "../commands.ts";
import {AppState, MediaSource, MediaState} from "../commands.ts";
import {Landing} from "./Landing.tsx";
import {useSources} from "../utils.tsx";
import {TabBar} from "./TabBar.tsx";
import {listen, UnlistenFn} from "@tauri-apps/api/event";

import './MediaSource.css'

export function MediaSourceController() {
    const sources = useSources();

    const [sourcesState, setSourcesState] = useState<Record<MediaSource, MediaState>>({});

    const changeSource = useCallback(async (source: MediaSource) => {
        const tabSourceState = sourcesState[source];
        console.log(`changing ${source}`, tabSourceState)
        await commands.switchSource(source)
    }, [sources, sourcesState]);


    useEffect(() => {
        let unlistenFn: UnlistenFn | null = null;
        (async () => {
            unlistenFn = await listen<AppState>('BACKEND_STATE_EVENT', (event) => {
                const {media} = event.payload;

                setSourcesState(media);
            });
            await commands.emitBackendState();
        })();

        //cleanup our listener
        return () => {
            unlistenFn && unlistenFn();
        };
    }, []);

    const currentSourceState = Object.values(sourcesState).find((s) => {
        if (s.type == 'multi') {
            return s.tabs.some((t) => t.isActive)
        } else {
            return s.tab?.isActive;
        }
    }) ?? null;

    return (<div className="media-source-controller">
        <MediaSourceBar
            currentSource={currentSourceState?.source}
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

interface MediaViewProps {
    sourceState: MediaState | null,
    changeSource: (source: MediaSource) => void
}

function MediaView({
                       sourceState,
                       changeSource,
                   }: MediaViewProps) {
    console.log(sourceState);
    if (sourceState) {
        console.log(sourceState)
        if (sourceState.type == 'multi') {
            return <TabBar
                source={sourceState.source}
                tabs={sourceState.tabs}
                onCreateTab={() => commands.createTab(sourceState.source)}
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