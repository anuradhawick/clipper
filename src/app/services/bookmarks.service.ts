import { inject, Injectable, OnDestroy, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { concatMap, delay, interval, Subscription } from "rxjs";
import { HistorySize, SettingsService } from "./settings.service";

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
  private historyManagementSubscription: Subscription | undefined;
  private settingsSubscription: Subscription | undefined;
  private unlistenBookmarkEntry: UnlistenFn | undefined;
  private unlistenBookmarkEvent: UnlistenFn | undefined;
  private settings: HistorySize = {
    clipboardHistorySize: 100,
    bookmarkHistorySize: 100,
  };
  private readonly settingsService = inject(SettingsService);

  constructor() {
    console.log("BookmarksService created");
    listen("bookmark_entry_added", (event: { payload: BookmarkEntry }) => {
      const currentEntries = this.items().filter(
        (e) => e.id !== event.payload.id,
      );
      this.items.set(
        [event.payload, ...currentEntries].slice(
          0,
          this.settings.bookmarkHistorySize,
        ),
      );
    }).then((func) => (this.unlistenBookmarkEntry = func));

    invoke<BookmarkEntry[]>("bookmarks_read_entries", {}).then((entries) => {
      this.items.set(entries);
    });

    // get user preference and override if different
    this.settingsSubscription = this.settingsService.settings$.subscribe(
      (saved: HistorySize) => {
        console.log("Bookmark settings updated", saved);
        this.settings = saved;
        invoke<BookmarkEntry[]>("bookmarks_read_entries", {}).then(
          (entries) => {
            this.items.set(entries);
          },
        );
      },
    );

    // clear old entries every 60 seconds, starts with a delay of 10 seconds
    this.historyManagementSubscription = interval(60000)
      .pipe(
        delay(10000),
        concatMap(async () => {
          await invoke<void>("bookmarks_clean_old_entries", {
            count: this.settings.bookmarkHistorySize,
          });
        }),
      )
      .subscribe();
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
    this.settingsSubscription && this.settingsSubscription.unsubscribe();
    this.historyManagementSubscription &&
      this.historyManagementSubscription.unsubscribe();
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
