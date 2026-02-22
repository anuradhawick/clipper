import { Component, computed, inject, Signal } from "@angular/core";
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

@Component({
  selector: "app-bookmarks-page",
  imports: [
    BookmarkItemComponent,
    MatIconModule,
    MatButtonModule,
    MatTooltipModule,
  ],
  templateUrl: "./bookmarks-page.component.html",
  styleUrl: "./bookmarks-page.component.scss",
})
export class BookmarksPageComponent {
  bookmarkEntries: Signal<BookmarkEntry[]>;
  readonly dialog = inject(MatDialog);

  constructor(protected bs: BookmarksService) {
    this.bookmarkEntries = computed(() => bs.items());
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
