<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  
  let Terminal: any;
  let FitAddon: any;
  let WebglAddon: any;
  
  export let sessionId: string | null = null;
  export let onClose: () => void = () => {};
  export let isFocused: boolean = false;
  
  let terminalElement: HTMLDivElement;
  let terminal: Terminal;
  let fitAddon: FitAddon;
  let unlistenOutput: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;
  let unlistenClosed: UnlistenFn | null = null;
  
  onMount(async () => {
    // Dynamically import xterm modules to avoid SSR issues
    const [xtermModule, fitModule, webglModule] = await Promise.all([
      import('@xterm/xterm'),
      import('@xterm/addon-fit'),
      import('@xterm/addon-webgl')
    ]);
    
    Terminal = xtermModule.Terminal;
    FitAddon = fitModule.FitAddon;
    WebglAddon = webglModule.WebglAddon;
    
    // Import CSS
    await import('@xterm/xterm/css/xterm.css');
    
    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'JetBrains Mono, Menlo, Monaco, Courier New, monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#d4d4d4',
        black: '#000000',
        red: '#cd3131',
        green: '#0dbc79',
        yellow: '#e5e510',
        blue: '#2472c8',
        magenta: '#bc3fbc',
        cyan: '#11a8cd',
        white: '#e5e5e5',
        brightBlack: '#666666',
        brightRed: '#f14c4c',
        brightGreen: '#23d18b',
        brightYellow: '#f5f543',
        brightBlue: '#3b8eea',
        brightMagenta: '#d670d6',
        brightCyan: '#29b8db',
        brightWhite: '#e5e5e5'
      }
    });
    
    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    
    try {
      const webglAddon = new WebglAddon();
      terminal.loadAddon(webglAddon);
    } catch (e) {
      console.warn('WebGL addon failed to load, falling back to canvas renderer');
    }
    
    terminal.open(terminalElement);
    fitAddon.fit();
    
    if (!sessionId) {
      sessionId = await invoke<string>('spawn_terminal', {
        cols: terminal.cols,
        rows: terminal.rows
      });
    }
    
    unlistenOutput = await listen<Uint8Array>(`terminal-output-${sessionId}`, (event) => {
      const data = new Uint8Array(event.payload);
      terminal.write(data);
    });
    
    unlistenError = await listen<string>(`terminal-error-${sessionId}`, (event) => {
      console.error('Terminal error:', event.payload);
      terminal.write(`\r\n\x1b[31mError: ${event.payload}\x1b[0m\r\n`);
    });
    
    unlistenClosed = await listen(`terminal-closed-${sessionId}`, () => {
      terminal.write('\r\n\x1b[33mTerminal session ended\x1b[0m\r\n');
      onClose();
    });
    
    terminal.onData((data) => {
      if (sessionId) {
        const encoder = new TextEncoder();
        const bytes = Array.from(encoder.encode(data));
        invoke('write_to_terminal', { sessionId, data: bytes }).catch(err => {
          console.error('Failed to write to terminal:', err);
        });
      }
    });
    
    // Focus the terminal immediately
    terminal.focus();
    
    terminal.onResize(({ cols, rows }) => {
      if (sessionId) {
        invoke('resize_terminal', { sessionId, cols, rows });
      }
    });
    
    window.addEventListener('resize', handleResize);
  });
  
  onDestroy(async () => {
    window.removeEventListener('resize', handleResize);
    
    if (unlistenOutput) await unlistenOutput();
    if (unlistenError) await unlistenError();
    if (unlistenClosed) await unlistenClosed();
    
    if (sessionId) {
      await invoke('close_terminal', { sessionId });
    }
    
    if (terminal) {
      terminal.dispose();
    }
  });
  
  function handleResize() {
    if (fitAddon) {
      fitAddon.fit();
    }
  }
  
  export function focus() {
    if (terminal) {
      terminal.focus();
    }
  }
  
  $: if (terminal && isFocused) {
    terminal.focus();
  }
</script>

<div 
  bind:this={terminalElement} 
  class="terminal-container"
  on:click={() => focus()}
  on:keydown={() => {}}
  role="button"
  tabindex="-1"
></div>

<style>
  .terminal-container {
    width: 100%;
    height: 100%;
    background-color: #1e1e1e;
  }
  
  :global(.xterm) {
    height: 100%;
    padding: 10px;
  }
</style>