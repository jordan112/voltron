import { writable } from 'svelte/store';

export interface TerminalTab {
  id: string;
  title: string;
}

export const terminals = writable<TerminalTab[]>([]);
export const activeTerminalId = writable<string | null>(null);

export function addTerminal(id: string, title: string) {
  terminals.update(tabs => [...tabs, { id, title }]);
  activeTerminalId.set(id);
}

export function removeTerminal(id: string) {
  terminals.update(tabs => {
    const filtered = tabs.filter(t => t.id !== id);
    return filtered;
  });
  
  activeTerminalId.update(currentId => {
    if (currentId === id) {
      const allTabs = get(terminals);
      return allTabs.length > 0 ? allTabs[allTabs.length - 1].id : null;
    }
    return currentId;
  });
}

import { get } from 'svelte/store';