import {useSources} from "../utils.tsx";

import "./MediaSourceBar.css";

interface MediaSourceBarProps {
    currentSource: MediaSource | null;
    changeSource: (id: MediaSource) => void;
}

export function MediaSourceBar({currentSource, changeSource}: MediaSourceBarProps) {
    const sources = useSources();

    return (
        <div className="media-source-bar">
            <div style={{flexShrink: 0, height: '100px'}}/>
            {[...sources.values()]
                .map((source) => (
                    <button
                        key={source.id}
                        onClick={() => changeSource(source.id)}
                        className={`media-source-add-button ${(currentSource == source.id) ? 'media-source-add-button--active' : ''}`}
                        title={`Open ${source.name}`}
                    >
                        <img
                            className="media-source-icon"
                            src={source.iconUrl}
                            alt=""
                            draggable={false}
                        />
                    </button>
                ))}
        </div>
    );
}
