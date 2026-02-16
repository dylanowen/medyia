import {createContext, useContext} from 'react';
import {MediaDefinition, MediaSource} from "./commands.ts";

export const MediaSourcesContext = createContext<Map<MediaSource, MediaDefinition>>(new Map());

export function useSources() {
    return useContext(MediaSourcesContext);
}