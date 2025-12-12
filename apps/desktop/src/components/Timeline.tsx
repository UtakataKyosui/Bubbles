import { createResource, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./Timeline.css";

type NostrEvent = {
    id: string;
    pubkey: string;
    created_at: number;
    kind: number;
    tags: string[][];
    content: string;
    sig: string;
};

const fetchTimeline = async (): Promise<NostrEvent[]> => {
    try {
        const eventsJson: string[] = await invoke("get_timeline", { limit: 20 });
        return eventsJson.map((e) => JSON.parse(e));
    } catch (error) {
        console.error("Failed to fetch timeline:", error);
        return [];
    }
};

export default function Timeline(props: { trigger?: () => number }) {
    const [timeline, { refetch }] = createResource(props.trigger || (() => 0), fetchTimeline);

    return (
        <div class="timeline-container">
            <h2>Timeline</h2>
            <button onClick={refetch}>Refresh</button>
            <div class="timeline-list">
                <For each={timeline()}>
                    {(event) => (
                        <div class="timeline-item">
                            <div class="event-header">
                                <span class="pubkey">{event.pubkey.slice(0, 8)}...</span>
                                <span class="time">
                                    {new Date(event.created_at * 1000).toLocaleString()}
                                </span>
                            </div>
                            <p class="event-content">{event.content}</p>
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
}
