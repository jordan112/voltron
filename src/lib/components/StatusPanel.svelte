<script lang="ts">
  export let mode: 'edit' | 'plan' = 'edit';
  export let isRunning: boolean = false;
  export let position: string;
  export let onModeChange: (mode: 'edit' | 'plan') => void;
  export let onCommand: (command: string) => void;
  
  function toggleMode() {
    const newMode = mode === 'edit' ? 'plan' : 'edit';
    onModeChange(newMode);
  }
  
  function sendCommand(cmd: string) {
    onCommand(cmd);
  }
</script>

<div class="status-panel">
  <div class="status-left">
    {#if isRunning}
      <div class="spinner"></div>
      <span>Running...</span>
    {/if}
  </div>
  
  <div class="status-center">
    <button class="cmd-btn" on:click={() => sendCommand('clear')} title="Clear terminal">
      Clear
    </button>
    <button class="cmd-btn" on:click={() => sendCommand('/compact')} title="Toggle compact mode">
      Compact
    </button>
    <button class="cmd-btn" on:click={() => sendCommand('/help')} title="Show help">
      Help
    </button>
    <button class="cmd-btn" on:click={() => sendCommand('/history')} title="Show history">
      History
    </button>
  </div>
  
  <div class="status-right">
    <button 
      class="mode-toggle"
      class:edit-mode={mode === 'edit'}
      class:plan-mode={mode === 'plan'}
      on:click={toggleMode}
    >
      {mode === 'edit' ? '✏️ Edit Mode' : '📋 Plan Mode'}
    </button>
  </div>
</div>

<style>
  .status-panel {
    height: 30px;
    background-color: #2d2d2d;
    border-top: 1px solid #3e3e3e;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px;
    font-size: 12px;
  }
  
  .status-left {
    flex: 0 0 auto;
    text-align: left;
    display: flex;
    align-items: center;
    gap: 8px;
    color: #ccc;
    min-width: 100px;
  }
  
  .status-center {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }
  
  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid #3e3e3e;
    border-top-color: #007acc;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .status-right {
    flex: 0 0 auto;
    text-align: right;
  }
  
  .cmd-btn {
    background: none;
    border: 1px solid #3e3e3e;
    color: #999;
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s;
    font-family: 'JetBrains Mono', monospace;
  }
  
  .cmd-btn:hover {
    background-color: #3e3e3e;
    border-color: #4e4e4e;
    color: #fff;
  }
  
  .cmd-btn:active {
    background-color: #4e4e4e;
    transform: scale(0.98);
  }
  
  .mode-toggle {
    background: none;
    border: 1px solid #3e3e3e;
    color: #ccc;
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s;
  }
  
  .mode-toggle:hover {
    background-color: #3e3e3e;
    border-color: #4e4e4e;
    color: #fff;
  }
  
  .mode-toggle.edit-mode {
    border-color: #0dbc79;
    color: #0dbc79;
  }
  
  .mode-toggle.plan-mode {
    border-color: #e5e510;
    color: #e5e510;
  }
</style>