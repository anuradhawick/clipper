import { Component, computed, Signal } from "@angular/core";
import { BookmarkItemComponent } from "./bookmark-item/bookmark-item.component";
import { MatIconModule } from "@angular/material/icon";
import { MatButtonModule } from "@angular/material/button";
import {
  BookmarkEntry,
  BookmarksService,
} from "../../../services/bookmarks.service";

@Component({
  selector: "app-bookmarks-page",
  imports: [BookmarkItemComponent],
  templateUrl: "./bookmarks-page.component.html",
  styleUrl: "./bookmarks-page.component.scss",
})
export class BookmarksPageComponent {
  bookmarkEntries: Signal<BookmarkEntry[]>;

  constructor(protected bs: BookmarksService) {
    this.bookmarkEntries = computed(() => bs.items());
    // this.bookmarkEntries = computed(() => [
    //   {
    //     id: "1",
    //     kind: ClipperEntryKind.Text,
    //     entry: "This is an inline test",
    //     timestamp: "2024-08-02T09:18:00.776Z",
    //   },
    //   {
    //     id: "2",
    //     kind: ClipperEntryKind.Text,
    //     entry:
    //       "This is an inline test for a very very very long one that might actually have some very ugly overflow",
    //     timestamp: "2024-08-02T09:18:00.776Z",
    //   },
    //   {
    //     id: "3",
    //     kind: ClipperEntryKind.Text,
    //     entry: `This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>`,
    //     timestamp: '2024-08-02T09:18:00.776Z'
    //   },
    //   {
    //     id: "4",
    //     kind: ClipperEntryKind.Text,
    //     entry: `This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>`,
    //     timestamp: '2024-08-02T09:18:00.776Z'
    //   },
    //   {
    //     id: "5",
    //     kind: ClipperEntryKind.Text,
    //     entry: `This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>`,
    //     timestamp: '2024-08-02T09:18:00.776Z'
    //   },
    // ]);
  }
}
