import { createSignal, Show } from "solid-js";
import "./App.css";
import Timeline from "./components/Timeline";
import PostNote from "./components/PostNote";
import BubbleBackground from "./components/BubbleBackground";

function App() {
  const [refreshTrigger, setRefreshTrigger] = createSignal(0);
  const [isModalOpen, setIsModalOpen] = createSignal(false);

  const handlePost = () => {
    setRefreshTrigger(t => t + 1);
    setIsModalOpen(false);
  };

  return (
    <>
      <BubbleBackground />
      <main class="container">
        <h1 class="app-title">Bubbles</h1>
        <Timeline trigger={refreshTrigger} />

        <button class="fab" onClick={() => setIsModalOpen(true)} aria-label="New Post">
          +
        </button>

        <Show when={isModalOpen()}>
          <div class="modal-overlay" onClick={(e) => {
            // Close if clicking the backdrop
            if (e.target === e.currentTarget) setIsModalOpen(false);
          }}>
            <div class="modal-content">
              <PostNote onPost={handlePost} />
              <button class="close-modal-btn" onClick={() => setIsModalOpen(false)}>Close</button>
            </div>
          </div>
        </Show>
      </main>
    </>
  );
}

export default App;
