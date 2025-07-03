<script lang="ts">
  import Terminal from '$lib/components/Terminal.svelte';
  import StatusPanel from '$lib/components/StatusPanel.svelte';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { browser } from '$app/environment';
  import { ask, open } from '@tauri-apps/plugin-dialog';
  
  interface TerminalPanel {
    id: string;
    position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
    mode: 'edit' | 'plan';
    isRunning: boolean;
  }
  
  let panels: TerminalPanel[] = [];
  let activePanel: string | null = null;
  let currentPath: string = '';
  let showPathHeader: boolean = false;
  
  async function createClaudeTerminal(position: string, workingDir: string) {
    if (!browser) return null;
    
    const id = await invoke<string>('spawn_terminal_with_command', {
      cols: 120,
      rows: 24,
      working_dir: workingDir,  // Fix parameter name to match Rust
      command: 'claude',
      args: []
    });
    return { id, position };
  }
  
  onMount(async () => {
    console.log('Component mounted, starting initialization...');
    
    try {
      // First, let's create basic terminals without dialog
      console.log('Creating basic terminals first...');
      const positions = ['top-left', 'top-right', 'bottom-left', 'bottom-right'];
      
      // Create regular terminals first to ensure basic functionality works
      const basicTerminalPromises = positions.map(async (pos) => {
        if (!browser) return null;
        try {
          const id = await invoke<string>('spawn_terminal', {
            cols: 120,
            rows: 24
          });
          console.log(`Created terminal ${id} at position ${pos}`);
          return { id, position: pos, mode: 'edit', isRunning: false };
        } catch (err) {
          console.error(`Failed to create terminal at ${pos}:`, err);
          return null;
        }
      });
      
      const basicTerminals = await Promise.all(basicTerminalPromises);
      panels = basicTerminals.filter(t => t !== null) as TerminalPanel[];
      console.log('Basic terminals created:', panels);
      
      // Set the first panel as active
      if (panels.length > 0) {
        activePanel = panels[0].id;
        
        // Now try to add Claude functionality
        setTimeout(async () => {
          try {
            console.log('Attempting to run Claude in terminals...');
            
            // Always prompt for directory
            let workingDir = '';
            
            try {
              const selected = await open({
                directory: true,
                multiple: false,
                title: 'Select Working Directory for Claude'
              });
              
              if (selected && typeof selected === 'string') {
                workingDir = selected;
                currentPath = selected;
                console.log('User selected directory:', workingDir);
              } else {
                console.log('User cancelled directory selection, using home directory');
                workingDir = await invoke<string>('get_home_dir');
                currentPath = workingDir;
              }
            } catch (err) {
              console.error('Error with directory picker:', err);
              // Use home directory as fallback
              workingDir = await invoke<string>('get_home_dir');
              currentPath = workingDir;
            }
            
            // Send command to run Claude in each terminal with unique ports
            // Use consistent ports based on position
            const portMap: Record<string, number> = {
              'top-left': 54545,
              'top-right': 54546,
              'bottom-left': 54547,
              'bottom-right': 54548
            };
            
            for (const panel of panels) {
              const port = portMap[panel.position];
              
              try {
                // Don't quote ~ to allow shell expansion, but quote other paths
                const cdPath = workingDir === '~' ? '~' : `"${workingDir}"`;
                
                // Send commands separately to avoid line wrapping issues
                // First, change directory
                const cdCommand = `cd ${cdPath}\n`;
                const cdEncoder = new TextEncoder();
                const cdBytes = Array.from(cdEncoder.encode(cdCommand));
                await invoke('write_to_terminal', { 
                  sessionId: panel.id,
                  data: cdBytes 
                });
                
                // Small delay to ensure cd completes
                await new Promise(resolve => setTimeout(resolve, 100));
                
                // Then run claude with the environment variable using full path
                const claudeCommand = `ANTHROPIC_OAUTH_PORT=${port} /Users/shane/.claude/local/claude\n`;
                const claudeEncoder = new TextEncoder();
                const claudeBytes = Array.from(claudeEncoder.encode(claudeCommand));
                await invoke('write_to_terminal', { 
                  sessionId: panel.id,
                  data: claudeBytes 
                });
                
                console.log(`Sent Claude command to terminal ${panel.position} with port ${port}`);
              } catch (err) {
                console.error(`Failed to send Claude command to terminal ${panel.id}:`, err);
              }
            }
          } catch (error) {
            console.error('Error setting up Claude:', error);
          }
        }, 2000); // Give terminals more time to initialize and resize
      }
    } catch (error) {
      console.error('Fatal error creating terminals:', error);
    }
  });
  
  function handlePanelClick(id: string) {
    activePanel = id;
  }
  
  function handleModeChange(panelId: string, newMode: 'edit' | 'plan') {
    panels = panels.map(p => 
      p.id === panelId ? { ...p, mode: newMode } : p
    );
    
    // Send command to Claude to switch modes
    const command = newMode === 'plan' ? '/plan\n' : '/chat\n';
    const encoder = new TextEncoder();
    const bytes = Array.from(encoder.encode(command));
    invoke('write_to_terminal', { 
      sessionId: panelId,
      data: bytes 
    }).catch(err => {
      console.error('Failed to send mode command:', err);
    });
  }
  
  function handleCommand(panelId: string, cmd: string) {
    const encoder = new TextEncoder();
    const command = cmd === 'clear' ? '\x1b[2J\x1b[H' : `${cmd}\n`;
    const bytes = Array.from(encoder.encode(command));
    invoke('write_to_terminal', { 
      sessionId: panelId,
      data: bytes 
    }).catch(err => {
      console.error('Failed to send command:', err);
    });
  }
  
  async function changePath() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select New Working Directory',
        defaultPath: currentPath
      });
      
      if (selected && typeof selected === 'string') {
        currentPath = selected;
        
        // Send cd command to all terminals
        for (const panel of panels) {
          const command = `cd "${selected}"\n`;
          const encoder = new TextEncoder();
          const bytes = Array.from(encoder.encode(command));
          await invoke('write_to_terminal', { 
            sessionId: panel.id,
            data: bytes 
          });
        }
      }
    } catch (err) {
      console.error('Error changing path:', err);
    }
  }
</script>

<div class="app-container">
  {#if currentPath}
    <div class="path-header">
      <span class="path-label">Working Directory:</span>
      <span class="path-value">{currentPath}</span>
      <button class="change-path-btn" on:click={changePath}>
        Change Path
      </button>
    </div>
  {/if}
  
  <div class="panel-grid">
    {#each panels as panel (panel.id)}
      <div 
        class="panel {panel.position}"
        class:active={activePanel === panel.id}
        on:click={() => handlePanelClick(panel.id)}
        on:keydown={() => {}}
        role="button"
        tabindex="0"
      >
        <div class="terminal-wrapper">
          <Terminal 
            sessionId={panel.id} 
            isFocused={activePanel === panel.id}
          />
        </div>
        <StatusPanel
          position={panel.position}
          mode={panel.mode}
          isRunning={panel.isRunning}
          onModeChange={(mode) => handleModeChange(panel.id, mode)}
          onCommand={(cmd) => handleCommand(panel.id, cmd)}
        />
      </div>
    {/each}
  </div>
</div>

<style>
  .app-container {
    height: 100vh;
    width: 100vw;
    background-color: #1e1e1e;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .path-header {
    height: 36px;
    background-color: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
    display: flex;
    align-items: center;
    padding: 0 15px;
    gap: 10px;
  }
  
  .path-label {
    color: #999;
    font-size: 12px;
    font-weight: 500;
  }
  
  .path-value {
    color: #ccc;
    font-size: 12px;
    flex: 1;
    font-family: 'JetBrains Mono', monospace;
  }
  
  .change-path-btn {
    background: none;
    border: 1px solid #3e3e3e;
    color: #999;
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.2s;
  }
  
  .change-path-btn:hover {
    background-color: #3e3e3e;
    border-color: #4e4e4e;
    color: #fff;
  }
  
  .panel-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    flex: 1;
    gap: 2px;
    background-color: #3e3e3e;
    padding: 2px;
  }
  
  .panel {
    background-color: #1e1e1e;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 2px solid transparent;
    transition: border-color 0.2s;
  }
  
  .panel.active {
    border-color: #007acc;
  }
  
  .panel:hover {
    border-color: #4e4e4e;
  }
  
  .panel.top-left {
    grid-column: 1;
    grid-row: 1;
  }
  
  .panel.top-right {
    grid-column: 2;
    grid-row: 1;
  }
  
  .panel.bottom-left {
    grid-column: 1;
    grid-row: 2;
  }
  
  .panel.bottom-right {
    grid-column: 2;
    grid-row: 2;
  }
  
  .terminal-wrapper {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>