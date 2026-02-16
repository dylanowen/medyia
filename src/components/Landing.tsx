import {MediaSource} from "../commands.ts";
import {useContext} from "react";
import {MediaSourcesContext} from "../utils.tsx";


interface LandingProps {
    onOpenService: (id: MediaSource) => void;
}

export function Landing({onOpenService}: LandingProps) {
    let sources = useContext(MediaSourcesContext);

    return (
        <div className="landing">
            <span className="landing-title">Open a service</span>
            <div className="landing-services">
                {[...sources.values()].map((source) => (
                    <button
                        key={source.id}
                        className="landing-service-button"
                        onClick={() => onOpenService(source.id)}
                    >
                        <img
                            className="landing-service-icon"
                            src={source.iconUrl}
                            alt=""
                            draggable={false}
                        />
                        {source.name}
                    </button>
                ))}
            </div>
        </div>
    );
}
