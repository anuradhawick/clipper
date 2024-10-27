import { Injectable, OnDestroy, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { HistorySize, SettingsService } from "./settings.service";
import { Subscription } from "rxjs";

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
  public items = signal<ClipperEntry[]>([]);
  public running = signal(true);
  private unlisten: UnlistenFn | undefined;
  private settingsSubscription: Subscription;
  private settings: HistorySize = { historySize: 100 };

  constructor(ss: SettingsService) {
    console.log("ClipboardHistoryService created");
    listen("clipboard_entry_added", (event: { payload: ClipperEntry }) => {
      this.items.update((entries) =>
        [event.payload, ...entries].slice(0, this.settings.historySize)
      );
    }).then((func) => (this.unlisten = func));

    // get user preference and override if different
    this.settingsSubscription = ss.settings$.subscribe((saved: HistorySize) => {
      console.log("Clipboard settings updated", saved);
      this.settings = saved;
      invoke<ClipperEntry[]>("read_clipboard_entries", {
        count: saved.historySize,
      }).then((entries) => {
        this.items.set(entries);
      });
    });
  }

  ngOnDestroy(): void {
    if (this.unlisten) {
      const unlisten = this.unlisten;
      unlisten();
    }
    this.settingsSubscription.unsubscribe();
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
