import {useEffect, useState} from "react";
import "./App.css";
import {getSources, MediaDefinition, MediaSource} from "./commands.ts";
import {MediaSourcesContext} from "./utils.tsx";
import {MediaSourceController} from "./components/MediaSource.tsx";

export default function App() {
    const [sources, setSources] = useState<Map<MediaSource, MediaDefinition>>(new Map());

    useEffect(() => {
        getSources().then((sources) => {
            // Our map should preserve insertion order
            const sourcesMap = new Map();
            for (const source of sources) {
                sourcesMap.set(source.id, source);
            }
            setSources(sourcesMap);
        });
    }, []);

    return (
        <MediaSourcesContext value={sources}>
            <MediaSourceController/>
        </MediaSourcesContext>
    );
}