import { Injectable, OnDestroy, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

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
    listen("bookmark_entry_added", (event: { payload: BookmarkEntry }) => {
      this.items.update((entries) => [event.payload, ...entries]);
    }).then((func) => (this.unlistenBookmarkEntry = func));

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
    // await invoke<void>("bookmarks_add_entry", {
    //   id,
    // });
  }

  async open(id: string) {
    // await invoke<void>("bookmarks_open_entry", { id });
  }

  async clear() {
    this.items.set([]);
    await invoke<void>("bookmarks_delete_all", {});
  }

  async delete(id: string) {
    this.items.update((entries) => entries.filter((e) => e.id != id));
    await invoke<void>("bookmarks_delete_one", { id });
  }
}
