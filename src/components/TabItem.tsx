import {TabState} from "../commands.ts";
import {useContext} from "react";
import {MediaSourcesContext} from "../utils.tsx";

interface TabItemProps {
    tab: TabState;
    isActive: boolean;
    onSelect: () => void;
    onClose: () => void;
}

export function TabItem({tab, isActive, onSelect, onClose}: TabItemProps) {
    const iconUrl = useContext(MediaSourcesContext).get(tab.source)!.iconUrl;

    return (
        <button
            className={`tab-item ${isActive ? "tab-item--active" : ""} ${tab.isPlaying ? "tab-item--playing" : ""}`}
            onClick={onSelect}
            title={tab.displayName}
        >
            {tab.isPlaying ? (
                <span className="tab-playing-icon">
          <span className="tab-playing-bar"/>
          <span className="tab-playing-bar"/>
          <span className="tab-playing-bar"/>
        </span>
            ) : (
                <img className="tab-icon" src={iconUrl} alt="" draggable={false}/>
            )}
            <span className="tab-name">{tab.displayName}</span>
            <span
                className="tab-close"
                onClick={(e) => {
                    e.stopPropagation();
                    onClose();
                }}
            >
        &times;
      </span>
        </button>
    );
}
