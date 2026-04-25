import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  inject,
  signal,
  Signal,
  viewChild,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatIconModule } from "@angular/material/icon";
import { MatInputModule } from "@angular/material/input";
import { MatTooltipModule } from "@angular/material/tooltip";
import { ClipboardItemComponent } from "../clipboard-page/clipboard-item/clipboard-item.component";
import {
  ClipperEntry,
  ClipperEntryKind,
  ClipboardHistoryService,
} from "../../../services/clipboard-history.service";
import { BookmarkItemComponent } from "../bookmarks-page/bookmark-item/bookmark-item.component";
import {
  BookmarkEntry,
  BookmarksService,
} from "../../../services/bookmarks.service";
import { NoteItemComponent } from "../notes-page/note-item/note-item.component";
import { NoteItem, NotesService } from "../../../services/notes.service";
import {
  TagEntry,
  TaggedItem,
  TaggedItemKind,
  TagsService,
} from "../../../services/tags.service";
import { asPlainText } from "../../../utils/text";

type TaggedManagerItem =
  | {
      kind: TaggedItemKind.Clipboard;
      item: ClipperEntry;
      tags: TagEntry[];
      latestTaggedAt: string;
    }
  | {
      kind: TaggedItemKind.Bookmark;
      item: BookmarkEntry;
      tags: TagEntry[];
      latestTaggedAt: string;
    }
  | {
      kind: TaggedItemKind.Note;
      item: NoteItem;
      tags: TagEntry[];
      latestTaggedAt: string;
    };

@Component({
  selector: "app-tagged-items-page",
  imports: [
    BookmarkItemComponent,
    ClipboardItemComponent,
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatTooltipModule,
    NoteItemComponent,
  ],
  templateUrl: "./tagged-items-page.component.html",
  styleUrl: "./tagged-items-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TaggedItemsPageComponent {
  protected readonly clipboardHistoryService = inject(ClipboardHistoryService);
  protected readonly bookmarksService = inject(BookmarksService);
  protected readonly notesService = inject(NotesService);
  protected readonly tagsService = inject(TagsService);
  protected readonly TaggedItemKind = TaggedItemKind;
  protected readonly tagColors = this.tagsService.tagColors;
  protected readonly showSearch = signal(false);
  protected readonly searchQuery = signal("");
  protected readonly selectedColorValues = signal<string[]>([]);
  private readonly searchInputRef =
    viewChild<ElementRef<HTMLInputElement>>("searchInput");

  protected readonly taggedItems: Signal<TaggedManagerItem[]> = computed(() => {
    const tagsById = new Map(
      this.tagsService.tags().map((tag) => [tag.id, tag] as const),
    );
    const assignmentsByItem = new Map<string, TaggedItem[]>();

    for (const assignment of this.tagsService.taggedItems()) {
      if (!tagsById.has(assignment.tag_id)) {
        continue;
      }

      const key = this.getItemKey(assignment.item_kind, assignment.item_id);
      const currentAssignments = assignmentsByItem.get(key) ?? [];
      currentAssignments.push(assignment);
      assignmentsByItem.set(key, currentAssignments);
    }

    const clipboardById = new Map(
      this.clipboardHistoryService.items().map((item) => [item.id, item]),
    );
    const bookmarksById = new Map(
      this.bookmarksService.items().map((item) => [item.id, item]),
    );
    const notesById = new Map(
      this.notesService.notes().map((item) => [item.id, item]),
    );
    const items: TaggedManagerItem[] = [];

    for (const assignments of assignmentsByItem.values()) {
      const firstAssignment = assignments[0];
      if (!firstAssignment) {
        continue;
      }

      const tags = assignments
        .map((assignment) => tagsById.get(assignment.tag_id))
        .filter((tag): tag is TagEntry => Boolean(tag));
      const latestTaggedAt = assignments.reduce(
        (latest, assignment) =>
          assignment.timestamp > latest ? assignment.timestamp : latest,
        firstAssignment.timestamp,
      );

      switch (firstAssignment.item_kind) {
        case TaggedItemKind.Clipboard: {
          const item = clipboardById.get(firstAssignment.item_id);
          if (item) {
            items.push({
              kind: TaggedItemKind.Clipboard,
              item,
              tags,
              latestTaggedAt,
            });
          }
          break;
        }
        case TaggedItemKind.Bookmark: {
          const item = bookmarksById.get(firstAssignment.item_id);
          if (item) {
            items.push({
              kind: TaggedItemKind.Bookmark,
              item,
              tags,
              latestTaggedAt,
            });
          }
          break;
        }
        case TaggedItemKind.Note: {
          const item = notesById.get(firstAssignment.item_id);
          if (item) {
            items.push({
              kind: TaggedItemKind.Note,
              item,
              tags,
              latestTaggedAt,
            });
          }
          break;
        }
      }
    }

    return items
      .filter((item) => this.matchesColorFilter(item))
      .filter((item) => this.matchesSearch(item, this.searchQuery()))
      .sort((left, right) =>
        right.latestTaggedAt.localeCompare(left.latestTaggedAt),
      );
  });

  protected readonly hasTaggedAssignments = computed(
    () => this.tagsService.taggedItems().length > 0,
  );

  protected readonly filterSummary = computed(() => {
    const selectedColorLabels = this.tagColors
      .filter((color) => this.selectedColorValues().includes(color.value))
      .map((color) => color.label);

    if (!selectedColorLabels.length) {
      return "Any tag";
    }

    return selectedColorLabels.join(", ");
  });

  protected toggleSearch(): void {
    const shouldShow = !this.showSearch();
    this.showSearch.set(shouldShow);

    if (shouldShow) {
      setTimeout(() => this.searchInputRef()?.nativeElement.focus());
    } else {
      this.searchQuery.set("");
    }
  }

  protected clearSearch(searchInput: HTMLInputElement): void {
    searchInput.value = "";
    this.searchQuery.set("");
  }

  protected toggleColor(kind: string): void {
    this.selectedColorValues.update((selectedColors) =>
      selectedColors.includes(kind)
        ? selectedColors.filter((selectedColor) => selectedColor !== kind)
        : [...selectedColors, kind],
    );
  }

  protected isColorSelected(kind: string): boolean {
    return this.selectedColorValues().includes(kind);
  }

  protected trackByTaggedItem(
    _: number,
    taggedItem: TaggedManagerItem,
  ): string {
    return this.getItemKey(taggedItem.kind, taggedItem.item.id);
  }

  protected asClipboardItem(taggedItem: TaggedManagerItem): ClipperEntry {
    return taggedItem.item as ClipperEntry;
  }

  protected asBookmarkItem(taggedItem: TaggedManagerItem): BookmarkEntry {
    return taggedItem.item as BookmarkEntry;
  }

  protected asNoteItem(taggedItem: TaggedManagerItem): NoteItem {
    return taggedItem.item as NoteItem;
  }

  private getItemKey(itemKind: TaggedItemKind, itemId: string): string {
    return `${itemKind}:${itemId}`;
  }

  private matchesColorFilter(item: TaggedManagerItem): boolean {
    const selectedColors = this.selectedColorValues();
    if (!selectedColors.length) {
      return true;
    }

    const itemColors = new Set(item.tags.map((tag) => tag.kind));
    return selectedColors.every((selectedColor) =>
      itemColors.has(selectedColor),
    );
  }

  private matchesSearch(item: TaggedManagerItem, query: string): boolean {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) {
      return true;
    }

    const tagText = item.tags
      .map((tag) => tag.tag)
      .join(" ")
      .toLowerCase();
    if (tagText.includes(normalizedQuery)) {
      return true;
    }

    switch (item.kind) {
      case TaggedItemKind.Clipboard:
        if (item.item.kind !== ClipperEntryKind.Text) {
          return false;
        }
        return asPlainText(item.item.entry)
          .toLowerCase()
          .includes(normalizedQuery);
      case TaggedItemKind.Bookmark:
        return (
          item.item.url.toLowerCase().includes(normalizedQuery) ||
          item.item.text.toLowerCase().includes(normalizedQuery)
        );
      case TaggedItemKind.Note:
        return item.item.entry.toLowerCase().includes(normalizedQuery);
    }

    return false;
  }
}
