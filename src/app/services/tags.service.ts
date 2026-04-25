import { Injectable, OnDestroy, signal } from "@angular/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { v4 as uuidv4 } from "uuid";

export enum TaggedItemKind {
  Clipboard = "clipboard",
  Bookmark = "bookmark",
  Note = "note",
}

export interface TagEntry {
  id: string;
  tag: string;
  kind: string;
  timestamp: string;
}

export interface TaggedItem {
  id: string;
  tag_id: string;
  item_kind: TaggedItemKind;
  item_id: string;
  timestamp: string;
}

export interface TagColorOption {
  value: string;
  label: string;
}

export const TAG_COLORS: TagColorOption[] = [
  { value: "slate", label: "Slate" },
  { value: "cyan", label: "Cyan" },
  { value: "blue", label: "Blue" },
  { value: "emerald", label: "Emerald" },
  { value: "amber", label: "Amber" },
  { value: "rose", label: "Rose" },
];

@Injectable({
  providedIn: "root",
})
export class TagsService implements OnDestroy {
  readonly tags = signal<TagEntry[]>([]);
  readonly taggedItems = signal<TaggedItem[]>([]);
  // Incremented when tag-item assignments change; components use it to refetch.
  readonly tagItemsVersion = signal(0);
  readonly tagColors = TAG_COLORS;
  private unlistenTagsEvent: UnlistenFn | undefined;
  private unlistenTagItemsEvent: UnlistenFn | undefined;

  constructor() {
    console.log("TagsService created");
    this.read();
    this.readTaggedItems();

    listen("tags_updated", async () => {
      await this.read();
    }).then((func) => (this.unlistenTagsEvent = func));

    // The event has no payload: it only invalidates visible item tag queries.
    listen("tag_items_updated", () => {
      this.tagItemsVersion.update((version) => version + 1);
      this.readTaggedItems();
    }).then((func) => (this.unlistenTagItemsEvent = func));
  }

  ngOnDestroy(): void {
    if (this.unlistenTagsEvent) {
      const unlisten = this.unlistenTagsEvent;
      unlisten();
    }
    if (this.unlistenTagItemsEvent) {
      const unlisten = this.unlistenTagItemsEvent;
      unlisten();
    }
  }

  async read() {
    const tags = await invoke<TagEntry[]>("tags_read_entries", {});
    this.tags.set(tags);
  }

  async readTaggedItems() {
    const taggedItems = await invoke<TaggedItem[]>("tags_read_items", {});
    this.taggedItems.set(taggedItems);
  }

  async create(tag: string, kind = "slate") {
    const trimmedTag = tag.trim();
    if (!trimmedTag) {
      return undefined;
    }

    const savedTag = await invoke<TagEntry>("tags_create_entry", {
      id: uuidv4(),
      tag: trimmedTag,
      kind,
    });
    this.tags.update((tags) => [
      savedTag,
      ...tags.filter((item) => item.id !== savedTag.id),
    ]);
    return savedTag;
  }

  async update(id: string, tag: string, kind: string) {
    const trimmedTag = tag.trim();
    if (!trimmedTag) {
      return;
    }

    const savedTag = await invoke<TagEntry>("tags_update_entry", {
      id,
      tag: trimmedTag,
      kind,
    });
    this.tags.update((tags) =>
      tags.map((item) => (item.id === id ? savedTag : item)),
    );
  }

  async delete(id: string) {
    await invoke<void>("tags_delete_one", { id });
    this.tags.update((tags) => tags.filter((item) => item.id !== id));
  }

  async readItemTags(itemKind: TaggedItemKind, itemId: string) {
    return invoke<TagEntry[]>("tags_read_item_tags", {
      itemKind,
      itemId,
    });
  }

  async setItemTags(
    itemKind: TaggedItemKind,
    itemId: string,
    tagIds: string[],
  ) {
    return invoke<TagEntry[]>("tags_set_item_tags", {
      itemKind,
      itemId,
      tagIds,
    });
  }
}
