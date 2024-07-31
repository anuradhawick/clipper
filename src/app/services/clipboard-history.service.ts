import { Injectable, OnDestroy, OnInit, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

export enum ClipperEntryKind {
  Text,
}

export interface ClipperEntry {
  id: string;
  entry: string;
  kind: ClipperEntryKind;
  timestamp: string;
}

@Injectable({
  providedIn: "root",
})
export class ClipboardHistoryService implements OnDestroy {
  unlisten: UnlistenFn | undefined;
  items = signal<ClipperEntry[]>([]);
  running = signal(true);

  constructor() {
    listen("clipboard_entry_added", (event: { payload: ClipperEntry }) => {
      this.items.update((entries) => [event.payload, ...entries].slice(0, 10));
    }).then((func) => (this.unlisten = func));

    invoke<ClipperEntry[]>("read_clipboard_entries", {}).then((entries) => {
      this.items.set(entries);
    });
  }

  ngOnDestroy(): void {
    if (this.unlisten) {
      const unlisten = this.unlisten;
      unlisten();
    }
  }

  async copy(index: number) {
    const entry = this.items().at(index);
    await invoke<void>("clipboard_add_entry", { entry: entry?.entry });
  }

  async pause() {
    this.running.set(false);
    await invoke<void>("pause_clipboard_watcher", {});
  }

  async resume() {
    this.running.set(true);
    await invoke<void>("resume_clipboard_watcher", {});
  }

  async clear() {
    this.items.set([]);
    await invoke<void>("delete_all_clipboard_entries", {});
  }

  async delete(id: string) {
    this.items.update((entries) => entries.filter((e) => e.id != id));
    await invoke<void>("delete_one_clipboard_entry", { id });
  }
}
