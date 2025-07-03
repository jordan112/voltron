<script lang="ts">
  import Terminal from '$lib/components/Terminal.svelte';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { browser } from '$app/environment';
  import { ask, open } from '@tauri-apps/plugin-dialog';
  import { Store } from '@tauri-apps/plugin-store';
  
  interface TerminalPanel {
    id: string;
    position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  }
  
  let panels: TerminalPanel[] = [];
  let activePanel: string | null = null;
  let store: Store;
  
  async function getWorkingDirectory(): Promise<string> {
    try {
      // Load saved path from store
      const savedPath = await store.get<string>('workingDirectory');
      console.log('Saved path from store:', savedPath);
      
      if (savedPath) {
        try {
          const useExisting = await ask(`Use previous directory?\n${savedPath}`, {
            title: 'Working Directory',
            okLabel: 'Yes',
            cancelLabel: 'Choose New'
          });
          
          if (useExisting) {
            return savedPath;
          }
        } catch (err) {
          console.error('Error showing ask dialog:', err);
        }
      }
      
      // Ask user to select a directory
      try {
        const selected = await open({
          directory: true,
          multiple: false,
          title: 'Select Working Directory for Claude',
          defaultPath: savedPath || undefined
        });
        
        if (selected && typeof selected === 'string') {
          await store.set('workingDirectory', selected);
          await store.save();
          return selected;
        }
      } catch (err) {
        console.error('Error showing directory picker:', err);
      }
      
      // If cancelled, use saved path or home directory
      if (savedPath) {
        return savedPath;
      }
      
      // Get home directory through invoke
      const homeDir = await invoke<string>('get_home_dir');
      return homeDir;
    } catch (error) {
      console.error('Error in getWorkingDirectory:', error);
      // Fallback to home directory
      return await invoke<string>('get_home_dir');
    }
  }
  
  async function createClaudeTerminal(position: string, workingDir: string) {
    if (!browser) return null;
    
    const id = await invoke<string>('spawn_terminal_with_command', {
      cols: 80,
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
            cols: 80,
            rows: 24
          });
          console.log(`Created terminal ${id} at position ${pos}`);
          return { id, position: pos };
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
            
            // For now, skip the store and just prompt for directory
            let workingDir = '~';
            
            try {
              const selected = await open({
                directory: true,
                multiple: false,
                title: 'Select Working Directory for Claude'
              });
              
              if (selected && typeof selected === 'string') {
                workingDir = selected;
                console.log('User selected directory:', workingDir);
              } else {
                console.log('User cancelled directory selection, using home directory');
              }
            } catch (err) {
              console.error('Error showing directory picker:', err);
              // Continue with home directory
            }
            
            // Send command to run Claude in each terminal
            for (const panel of panels) {
              try {
                const command = `cd "${workingDir}" && claude\n`;
                const encoder = new TextEncoder();
                const bytes = Array.from(encoder.encode(command));
                await invoke('write_to_terminal', { 
                  sessionId: panel.id,  // Fix parameter name
                  data: bytes 
                });
                console.log(`Sent Claude command to terminal ${panel.id}`);
              } catch (err) {
                console.error(`Failed to send Claude command to terminal ${panel.id}:`, err);
              }
            }
          } catch (error) {
            console.error('Error setting up Claude:', error);
          }
        }, 1000); // Give terminals time to initialize
      }
    } catch (error) {
      console.error('Fatal error creating terminals:', error);
    }
  });
  
  function handlePanelClick(id: string) {
    activePanel = id;
  }
</script>

<div class="app-container">
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
  }
  
  .panel-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    height: 100%;
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
  }
</style>