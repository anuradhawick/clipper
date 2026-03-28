import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  inject,
  input,
  output,
  signal,
  viewChild,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { DatePipe } from "@angular/common";
import { openUrl } from "@tauri-apps/plugin-opener";
import { MatMenuModule, MatMenuTrigger } from "@angular/material/menu";
import { WindowActionsService } from "../../../../services/window-actions.service";
import { BookmarkEntry } from "../../../../services/bookmarks.service";
import { MatDialog } from "@angular/material/dialog";
import {
  BookmarkItemDialogComponent,
  BookmarkItemDialogData,
} from "./bookmark-item-dialog.component";

const ITEM_HEIGHT_PX = 140;

@Component({
  selector: "app-bookmark-item",
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: "relative block w-full min-w-0 pb-1",
    "[style.height.px]": "itemHeightPx",
  },
  imports: [MatButtonModule, MatIconModule, DatePipe, MatMenuModule],
  templateUrl: "./bookmark-item.component.html",
  styleUrl: "./bookmark-item.component.scss",
  providers: [],
})
export class BookmarkItemComponent {
  bookmarkEntry = input.required<BookmarkEntry>();
  deleteClicked = output();
  copyClicked = output();
  openClicked = output();
  refreshClicked = output();
  clickedUrl = signal("");
  menu = viewChild.required<MatMenuTrigger>(MatMenuTrigger);
  contextMenuPosition = { x: "0px", y: "0px" };
  openUrl = openUrl;
  readonly itemHeightPx = ITEM_HEIGHT_PX;
  readonly hostElement = inject<ElementRef<HTMLElement>>(ElementRef);
  readonly windowService = inject(WindowActionsService);
  readonly dialog = inject(MatDialog);

  processImage(image: Array<number>): string {
    const bytes = Uint8Array.from(image);
    const blob = new Blob([bytes], { type: "image" });
    const url = URL.createObjectURL(blob);

    return url;
  }

  onLinkRightClick(event: MouseEvent, url: string) {
    event.preventDefault();

    const hostRect = this.hostElement.nativeElement.getBoundingClientRect();

    this.contextMenuPosition.x = `${event.clientX - hostRect.left}px`;
    this.contextMenuPosition.y = `${event.clientY - hostRect.top}px`;
    this.clickedUrl.set(url);
    this.menu().openMenu();
  }

  showQRCode(url = this.clickedUrl()) {
    this.windowService.openQrViewer(url);
  }

  openExpandedView() {
    this.dialog.open<BookmarkItemDialogComponent, BookmarkItemDialogData>(
      BookmarkItemDialogComponent,
      {
        data: {
          bookmarkEntry: this.bookmarkEntry(),
        },
        width: "100vw",
        height: "100vh",
        maxWidth: "100vw",
        maxHeight: "100vh",
        autoFocus: false,
        panelClass: "clipper-fullscreen-dialog-panel",
      },
    );
  }
}
