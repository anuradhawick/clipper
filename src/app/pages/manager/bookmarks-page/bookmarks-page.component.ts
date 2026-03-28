import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  Signal,
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
  ],
  templateUrl: "./bookmarks-page.component.html",
  styleUrl: "./bookmarks-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BookmarksPageComponent {
  bookmarkEntries: Signal<BookmarkEntry[]>;
  readonly dialog = inject(MatDialog);
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;

  constructor(protected bs: BookmarksService) {
    this.bookmarkEntries = computed(() => bs.items());
  }

  protected trackByBookmarkId(_: number, bookmarkEntry: BookmarkEntry): string {
    return bookmarkEntry.id;
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
