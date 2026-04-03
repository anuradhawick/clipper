import { inject, Injectable, OnDestroy, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { HistorySize, SettingsService } from "./settings.service";
import { Subscription } from "rxjs";

export enum ClipperEntryKind {
  Text = "Text",
  Image = "Image",
}

export interface ClipperEntry {
  id: string;
  entry: Array<number>;
  kind: ClipperEntryKind;
  timestamp: string;
}

@Injectable({
  providedIn: "root",
})
export class ClipboardHistoryService implements OnDestroy {
  public items = signal<ClipperEntry[]>([]);
  public running = signal(false);
  private unlistenClipboardEntry: UnlistenFn | undefined;
  private unlistenClipboardStatus: UnlistenFn | undefined;
  private unlistenClipboardEvent: UnlistenFn | undefined;
  private settingsSubscription: Subscription | undefined;
  private settings: HistorySize = {
    clipboardHistorySize: 100,
    bookmarkHistorySize: 100,
  };
  private readonly settingsService = inject(SettingsService);

  constructor() {
    console.log("ClipboardHistoryService created");
    listen("clipboard_entry_added", (event: { payload: ClipperEntry }) => {
      this.items.update((entries) =>
        [event.payload, ...entries].slice(
          0,
          this.settings.clipboardHistorySize,
        ),
      );
    }).then((func) => (this.unlistenClipboardEntry = func));

    listen("clipboard_status_changed", (event: { payload: boolean }) => {
      this.running.set(event.payload);
    }).then((func) => (this.unlistenClipboardStatus = func));

    listen("clipboard_updated", async () => {
      const entries = await invoke<ClipperEntry[]>("clipboard_read_entries", {
        count: this.settings.clipboardHistorySize,
      });
      this.items.set(entries);
    }).then((func) => (this.unlistenClipboardEvent = func));

    // get user preference and override if different
    this.settingsSubscription = this.settingsService.settings$.subscribe(
      (saved: HistorySize) => {
        console.log("Clipboard settings updated", saved);
        this.settings = saved;
        invoke<ClipperEntry[]>("clipboard_read_entries", {
          count: saved.clipboardHistorySize,
        }).then((entries) => {
          this.items.set(entries);
        });
      },
    );

    invoke<boolean>("clipboard_read_status", {}).then((running) => {
      this.running.set(running);
    });
  }

  ngOnDestroy(): void {
    if (this.unlistenClipboardEntry) {
      const unlisten = this.unlistenClipboardEntry;
      unlisten();
    }
    if (this.unlistenClipboardEvent) {
      const unlisten = this.unlistenClipboardEvent;
      unlisten();
    }
    if (this.unlistenClipboardStatus) {
      const unlisten = this.unlistenClipboardStatus;
      unlisten();
    }
    this.settingsSubscription && this.settingsSubscription.unsubscribe();
  }

  async copy(id: string) {
    await invoke<void>("clipboard_add_entry", {
      id,
    });
  }

  async open(id: string) {
    await invoke<void>("clipboard_open_entry", { id });
  }

  async pause() {
    this.running.set(false);
    await invoke<void>("clipboard_pause_watcher", {});
  }

  async resume() {
    this.running.set(true);
    await invoke<void>("clipboard_resume_watcher", {});
  }

  async clear() {
    this.items.set([]);
    await invoke<void>("clipboard_delete_all_entries", {});
  }

  async delete(id: string) {
    this.items.update((entries) => entries.filter((e) => e.id != id));
    await invoke<void>("clipboard_delete_one_entry", { id });
  }
}
