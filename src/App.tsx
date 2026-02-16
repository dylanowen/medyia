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

// function AppOld() {
//     const [sources, setSources] = useState<Map<MediaSource, MediaDefinition>>(new Map());
//
//     const [currentSource, setCurrentSource] = useState<MediaSource | null>(null)
//
//     const [tabs, setTabs] = useState<TabState[]>([]);
//     const [activeTab, setActiveTab] = useState<MediaSource | null>(null);
//
//     console.log(currentSource);
//
//     const refreshTabs = useCallback(async () => {
//         const [tabList, active] = await getTabs()
//         console.log("[medya] tabs refreshed:", tabList);
//         setTabs(tabList);
//         setActiveTab(active);
//     }, []);
//
//     useEffect(() => {
//         console.log("[medya] App mounted, setting up listeners");
//         getSources().then((sources) => {
//             const sourcesMap = new Map();
//             for (const source of sources) {
//                 sourcesMap.set(source.id, source);
//             }
//             setSources(sourcesMap);
//         });
//         refreshTabs();
//
//         const unlisten = listen("playback-changed", () => {
//             console.log("[medya] playback-changed event received");
//             debug("[medya] playback-changed event received");
//             refreshTabs();
//         });
//
//         unlisten.then(() => console.log("[medya] playback-changed listener registered"));
//
//         return () => {
//             unlisten.then((fn) => fn());
//         };
//     }, [refreshTabs]);
//
//     const handleCreateTab = async (source: MediaSource) => {
//         await createTab(source);
//         await refreshTabs();
//     };
//
//     const handleSwitchTab = async (key: string) => {
//         await switchTab(key)
//         await refreshTabs();
//     };
//
//     const handleCloseTab = async (key: string) => {
//         await closeTab(key)
//         await refreshTabs();
//     };
//
//     return (
//         <MediaSourcesContext value={sources}>
//             <div className="app">
//                 <MediaSourceBar
//                     currentSource={currentSource}
//                     changeSource={setCurrentSource}
//                 />
//                 <div>
//                     <div className="media-source">
//                         {tabs.length === 0 && (
//                             <Landing onOpenService={handleCreateTab}/>
//                         )}
//                     </div>
//                     <TabBar
//                         tabs={tabs}
//                         activeTab={activeTab}
//                         onCreateTab={handleCreateTab}
//                         onSwitchTab={handleSwitchTab}
//                         onCloseTab={handleCloseTab}
//                     />
//                 </div>
//
//             </div>
//         </MediaSourcesContext>
//     );
// }
//
// // export default AppOld;
