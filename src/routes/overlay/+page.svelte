<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  let title = $page.url.searchParams.get('title') || 'Limit Reached';
  let body = $page.url.searchParams.get('body') || 'Your time is up.';
  let isVisible = true;

  let timeoutId: ReturnType<typeof setTimeout>;

  function startTimer() {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(closeOverlay, 5000);
  }

  onMount(async () => {
    startTimer();

    listen('overlay_update', (event: any) => {
      isVisible = true;
      title = event.payload.title;
      body = event.payload.body;
      startTimer();
    });

    listen('rule_countdown', (event: any) => {
      if (!isVisible) return;
      let r = event.payload.remainingSeconds;
      let target = event.payload.target;
      let enf = event.payload.enforcement;

      if (enf === 'hard') {
        title = r <= 0 ? "Focus Block: Closed" : `Hard Block in ${r}s`;
        body = r <= 0 ? `Time limit exceeded. ${target} has been closed.` : `Save your work immediately! ${target} is closing.`;
      } else if (enf === 'warn') {
        if (r <= 0) {
          title = "Limit Reached";
          body = `Limit reached! Closed ${target}. Extend to keep using.`;
        } else {
          let m = Math.floor(r / 60);
          title = m > 0 ? `Remaining: ${m}m` : `Remaining: ${r}s`;
          body = `Warning: Focus limit is approaching for ${target}. Extend time to keep active.`;
        }
      }

      if (r <= 10) {
        if (timeoutId) clearTimeout(timeoutId);
      }
    });
  });

  async function closeOverlay() {
    isVisible = false;
    try {
      await invoke('close_overlay');
    } catch (e) {
      console.error("Failed to invoke close_overlay:", e);
    }
  }
</script>

<div class="overlay-container">
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="toast-content" onclick={closeOverlay}>
    <div class="icon-container">
      <svg class="lock-icon" viewBox="0 0 24 24">
        <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"/>
      </svg>
    </div>
    
    <div class="text-container">
      <h1>{title}</h1>
      <p class="description">{body}</p>
    </div>
    
    <button class="close-btn" aria-label="Close" onclick={(e) => { e.stopPropagation(); closeOverlay(); }}>
      <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  </div>
</div>

<style>
  :global(body) {
    background-color: transparent !important;
    margin: 0;
    overflow: hidden;
  }

  .overlay-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px;
    box-sizing: border-box;
    font-family: 'Outfit', sans-serif;
    color: #f3f4f6;
  }

  .toast-content {
    width: 100%;
    height: 100%;
    background-color: rgba(17, 24, 39, 0.95);
    border: 1px solid #ef4444;
    padding: 16px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    gap: 16px;
    box-shadow: 0 4px 24px rgba(239, 68, 68, 0.2);
    animation: slideIn 0.3s cubic-bezier(0.16, 1, 0.3, 1) both;
    cursor: pointer;
    position: relative;
    box-sizing: border-box;
  }

  .icon-container {
    width: 48px;
    height: 48px;
    flex-shrink: 0;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.25);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ef4444;
    animation: pulseRed 2s infinite ease-in-out;
  }

  .lock-icon {
    width: 24px;
    height: 24px;
    fill: currentColor;
  }

  .text-container {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  h1 {
    font-size: 16px;
    font-weight: 700;
    margin: 0 0 4px 0;
    color: #ffffff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  p.description {
    font-size: 14px;
    color: #9ca3af;
    line-height: 1.4;
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #9ca3af;
    cursor: pointer;
    padding: 8px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    align-self: flex-start;
    margin-top: -4px;
    margin-right: -4px;
  }

  .close-btn:hover {
    color: #ffffff;
    background-color: rgba(255, 255, 255, 0.1);
  }

  @keyframes slideIn {
    from { transform: translateX(20px); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  @keyframes pulseRed {
    0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
    50% { box-shadow: 0 0 10px 2px rgba(239, 68, 68, 0.2); }
  }
</style>
