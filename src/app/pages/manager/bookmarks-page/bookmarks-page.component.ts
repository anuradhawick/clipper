import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  inject,
  Signal,
  signal,
  viewChild,
  ViewChild,
} from "@angular/core";
import { ScrollingModule } from "@angular/cdk/scrolling";
import { BookmarkItemComponent } from "./bookmark-item/bookmark-item.component";
import { MatIconModule } from "@angular/material/icon";
import { MatButtonModule } from "@angular/material/button";
import {
  BookmarkEntry,
  BookmarksService,
} from "../../../services/bookmarks.service";
import { MatDialog } from "@angular/material/dialog";
import { ActionConfirmationDialogComponent } from "../../../components/action-confirmation-dialog/action-confirmation-dialog.component";
import { MatTooltipModule } from "@angular/material/tooltip";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";

const ITEM_HEIGHT_PX = 140;
const MIN_BUFFER_PX = 280;
const MAX_BUFFER_PX = 560;

@Component({
  selector: "app-bookmarks-page",
  imports: [
    BookmarkItemComponent,
    ScrollingModule,
    MatIconModule,
    MatButtonModule,
    MatTooltipModule,
    MatFormFieldModule,
    MatInputModule,
  ],
  templateUrl: "./bookmarks-page.component.html",
  styleUrl: "./bookmarks-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BookmarksPageComponent {
  private searchInputRef =
    viewChild<ElementRef<HTMLInputElement>>("searchInput");
  protected bookmarkEntries: Signal<BookmarkEntry[]>;
  protected readonly dialog = inject(MatDialog);
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;
  protected showSearch = signal(false);
  protected searchQuery = signal("");

  constructor(protected bs: BookmarksService) {
    this.bookmarkEntries = computed(() =>
      bs
        .items()
        .filter((entry) => this.matchesSearch(entry, this.searchQuery())),
    );
  }

  protected trackByBookmarkId(_: number, bookmarkEntry: BookmarkEntry): string {
    return bookmarkEntry.id;
  }

  protected toggleSearch(): void {
    const shouldShow = !this.showSearch();
    this.showSearch.set(shouldShow);

    if (shouldShow) {
      setTimeout(() => this.searchInputRef()?.nativeElement.focus());
    } else {
      this.searchQuery.set("");
    }
  }

  protected clearSearch(searchInput: HTMLInputElement) {
    searchInput.value = "";
    this.searchQuery.set("");
  }

  private matchesSearch(entry: BookmarkEntry, query: string): boolean {
    if (!query) {
      return true;
    }

    const normalizedQuery = query.toLowerCase();
    return (
      entry.url.toLowerCase().includes(normalizedQuery) ||
      entry.text.toLowerCase().includes(normalizedQuery)
    );
  }

  clearBookmarks() {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Clear Bookmarks`,
        message: `Are you sure you want to clear all bookmarks?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.bs.clear();
      }
    });
  }
}
