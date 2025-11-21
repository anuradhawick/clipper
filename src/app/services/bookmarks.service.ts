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

export interface BookmarkEntry {
  id: string;
  url: string;
  text: string;
  image: Array<number> | null;
  timestamp: string;
}

@Injectable({
  providedIn: "root",
})
export class BookmarksService implements OnDestroy {
  public items = signal<BookmarkEntry[]>([]);
  private unlistenBookmarkEntry: UnlistenFn | undefined;
  private unlistenBookmarkEvent: UnlistenFn | undefined;

  constructor() {
    console.log("BookmarksService created");
    // listen("clipboard_entry_added", (event: { payload: BookmarkEntry }) => {
    //   this.items.update((entries) =>
    //     [event.payload, ...entries].slice(0, this.settings.historySize)
    //   );
    // }).then((func) => (this.unlistenBookmarkEntry = func));

    invoke<BookmarkEntry[]>("bookmarks_read_entries", {}).then((entries) => {
      this.items.set(entries);
    });
  }

  ngOnDestroy(): void {
    if (this.unlistenBookmarkEntry) {
      const unlisten = this.unlistenBookmarkEntry;
      unlisten();
    }
    if (this.unlistenBookmarkEvent) {
      const unlisten = this.unlistenBookmarkEvent;
      unlisten();
    }
  }

  async copy(id: string) {
    await invoke<void>("clipboard_add_entry", {
      id,
    });
  }

  async open(id: string) {
    await invoke<void>("clipboard_open_entry", { id });
  }

  async clear() {
    this.items.set([]);
    // await invoke<void>("clipboard_delete_all_entries", {});
  }

  async delete(id: string) {
    this.items.update((entries) => entries.filter((e) => e.id != id));
    await invoke<void>("delete_bookmark", { id });
  }
}
