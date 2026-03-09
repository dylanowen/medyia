import {TabItem} from "./TabItem";
import {MediaSource, TabKey, TabState} from "../commands.ts";
import {useContext} from "react";
import {MediaSourcesContext} from "../utils.tsx";

interface TabBarProps {
    source: MediaSource,
    tabs: TabState[],
    onCreateTab: () => void;
    onSwitchTab: (key: TabKey) => void;
    onCloseTab: (key: TabKey) => void;
}

export function TabBar({
                           source,
                           tabs,
                           onCreateTab,
                           onSwitchTab,
                           onCloseTab,
                       }: TabBarProps) {
    const sourceDefinition = useContext(MediaSourcesContext).get(source)!;

    return (
        <div className="tab-bar">
            <div className="tab-bar-tabs">
                {tabs.map((tab) => (
                    <TabItem
                        key={tab.key}
                        tab={tab}
                        onSelect={() => onSwitchTab(tab.key)}
                        onClose={() => onCloseTab(tab.key)}
                    />
                ))}
            </div>
            <div className="tab-bar-actions">
                <button
                    key={sourceDefinition.id}
                    className="tab-add-button"
                    onClick={onCreateTab}
                    title={`Open ${sourceDefinition.name}`}
                >
                    <img
                        className="tab-icon"
                        src={sourceDefinition.iconUrl}
                        alt=""
                        draggable={false}
                    />
                    <span className="tab-add-plus">+</span>
                </button>
            </div>
        </div>
    );
}
