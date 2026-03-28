import {
  ChangeDetectionStrategy,
  Component,
  ViewEncapsulation,
  inject,
} from "@angular/core";
import { DatePipe } from "@angular/common";
import { MatButtonModule } from "@angular/material/button";
import { MatDialogClose, MAT_DIALOG_DATA } from "@angular/material/dialog";
import { MatIconModule } from "@angular/material/icon";
import { openUrl } from "@tauri-apps/plugin-opener";
import { BookmarkEntry } from "../../../../services/bookmarks.service";

export interface BookmarkItemDialogData {
  bookmarkEntry: BookmarkEntry;
}

@Component({
  selector: "app-bookmark-item-dialog",
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  imports: [DatePipe, MatButtonModule, MatDialogClose, MatIconModule],
  templateUrl: "./bookmark-item-dialog.component.html",
  styleUrl: "./bookmark-item-dialog.component.scss",
})
export class BookmarkItemDialogComponent {
  readonly data = inject<BookmarkItemDialogData>(MAT_DIALOG_DATA);
  readonly openUrl = openUrl;

  processImage(image: Array<number>): string {
    const bytes = Uint8Array.from(image);
    const blob = new Blob([bytes], { type: "image" });

    return URL.createObjectURL(blob);
  }
}
