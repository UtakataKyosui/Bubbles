import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./PostNote.css";

export default function PostNote(props: { onPost: () => void }) {
    const [content, setContent] = createSignal("");
    const [status, setStatus] = createSignal("");

    const publish = async (e: Event) => {
        e.preventDefault();
        if (!content()) return;

        try {
            setStatus("Publishing...");
            await invoke("publish_note", { content: content() });
            setContent("");
            setStatus("Published!");
            props.onPost();
            setTimeout(() => setStatus(""), 3000);
        } catch (error) {
            console.error("Failed to publish:", error);
            setStatus(`Error: ${error}`);
        }
    };

    return (
        <div class="post-note-container">
            <h3>New Note</h3>
            <form onSubmit={publish}>
                <textarea
                    value={content()}
                    onInput={(e) => setContent(e.currentTarget.value)}
                    placeholder="What's on your mind?"
                    rows={3}
                />
                <div class="form-actions">
                    <span class="status">{status()}</span>
                    <button type="submit" disabled={!content()}>
                        Post
                    </button>
                </div>
            </form>
        </div>
    );
}
