// Base media monitor - injected into all streaming service webviews
// TAB_LABEL and SOURCE_ID are set by the wrapper in scripts.rs

(function setupMediaMonitor() {
  const POLL_INTERVAL = 2000;
  let trackedElements = new WeakSet();
  let lastState = null;

  function getMetadata() {
    // Service-specific metadata extractors override this via window.__medyia_getMetadata
    if (typeof window.__medyia_getMetadata === 'function') {
      return window.__medyia_getMetadata();
    }
    return { title: document.title, artist: null, artworkUrl: null };
  }

  function emitState(playing) {
    const metadata = getMetadata();
    const state = {
      label: TAB_LABEL,
      playing: playing,
      title: metadata.title || null,
      artist: metadata.artist || null,
      artworkUrl: metadata.artworkUrl || null,
    };

    const stateKey = JSON.stringify(state);
    if (stateKey === lastState) return;
    lastState = stateKey;

    if (window.__TAURI__) {
      window.__TAURI__.event.emit('playback-state', state);
    }
  }

  function bindMediaElement(el) {
    if (trackedElements.has(el)) return;
    trackedElements.add(el);

    el.addEventListener('play', () => emitState(true));
    el.addEventListener('pause', () => emitState(false));
    el.addEventListener('ended', () => emitState(false));
  }

  function scanForMedia() {
    document.querySelectorAll('video, audio').forEach(bindMediaElement);
  }

  // Initial scan
  scanForMedia();

  // Watch for dynamically added media elements
  const observer = new MutationObserver(() => {
    scanForMedia();
  });
  observer.observe(document.documentElement, {
    childList: true,
    subtree: true,
  });

  // Fallback polling
  setInterval(() => {
    scanForMedia();
    const mediaElements = document.querySelectorAll('video, audio');
    let anyPlaying = false;
    mediaElements.forEach((el) => {
      if (!el.paused && !el.ended) {
        anyPlaying = true;
      }
    });
    // Only emit on poll if state changed
    emitState(anyPlaying);
  }, POLL_INTERVAL);

  // Tab title observer — emits document.title changes to Rust
  let lastDocTitle = '';
  function emitTitleChange() {
    const title = document.title;
    if (title && title !== lastDocTitle) {
      lastDocTitle = title;
      if (window.__TAURI__) {
        window.__TAURI__.event.emit('tab-title-changed', {
          label: TAB_LABEL,
          title: title,
        });
      }
    }
  }

  // Observe <title> element mutations
  const titleEl = document.querySelector('title');
  if (titleEl) {
    new MutationObserver(emitTitleChange).observe(titleEl, { childList: true });
  }
  // Fallback poll for title changes
  setInterval(emitTitleChange, 3000);
})();
