import { inject, Injectable, OnDestroy, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { HistorySize, SettingsService } from "./settings.service";
import { delay, interval, Subscription } from "rxjs";
import { concatMap } from "rxjs/operators";

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
  private unlistenClipboardEvent: UnlistenFn | undefined;
  private settingsSubscription: Subscription | undefined;
  private historyManagementSubscription: Subscription | undefined;
  private settings: HistorySize = { historySize: 100 };
  private readonly settingsService = inject(SettingsService);

  constructor() {
    console.log("ClipboardHistoryService created");
    listen("clipboard_entry_added", (event: { payload: ClipperEntry }) => {
      this.items.update((entries) =>
        [event.payload, ...entries].slice(0, this.settings.historySize),
      );
    }).then((func) => (this.unlistenClipboardEntry = func));

    listen("clipboard_status_changed", (event: { payload: boolean }) => {
      this.running.set(event.payload);
    }).then((func) => (this.unlistenClipboardEntry = func));

    // get user preference and override if different
    this.settingsSubscription = this.settingsService.settings$.subscribe(
      (saved: HistorySize) => {
        console.log("Clipboard settings updated", saved);
        this.settings = saved;
        invoke<ClipperEntry[]>("clipboard_read_entries", {
          count: saved.historySize,
        }).then((entries) => {
          this.items.set(entries);
        });
      },
    );

    invoke<boolean>("clipboard_read_status", {}).then((running) => {
      this.running.set(running);
    });

    // clear old entries every 60 seconds, starts with a delay of 10 seconds
    this.historyManagementSubscription = interval(60000)
      .pipe(
        delay(10000),
        concatMap(async () => {
          await invoke<void>("clipboard_clean_old_entries", {
            count: this.settings.historySize,
          });
        }),
      )
      .subscribe();
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
    this.settingsSubscription && this.settingsSubscription.unsubscribe();
    this.historyManagementSubscription &&
      this.historyManagementSubscription.unsubscribe();
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
