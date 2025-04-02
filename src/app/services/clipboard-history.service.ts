import { Injectable, OnDestroy, signal } from "@angular/core";
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
  public running = signal(true);
  private unlisten: UnlistenFn | undefined;
  private settingsSubscription: Subscription;
  private historyManagementSubscription: Subscription;
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

    // clear old entries every 60 seconds, starts with a delay of 10 seconds
    this.historyManagementSubscription = interval(60000)
      .pipe(
        delay(10000),
        concatMap(async () => {
          await invoke<void>("clean_old_entries", {
            count: this.settings.historySize,
          });
        })
      )
      .subscribe();
  }

  ngOnDestroy(): void {
    if (this.unlisten) {
      const unlisten = this.unlisten;
      unlisten();
    }
    this.settingsSubscription.unsubscribe();
    this.historyManagementSubscription.unsubscribe();
  }

  async copy(id: string) {
    await invoke<void>("clipboard_add_entry", {
      id,
    });
  }

  async open(id: string) {
    await invoke<void>("open_clipboard_entry", { id });
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
