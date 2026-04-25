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
import { MatDialog } from "@angular/material/dialog";
import { MatIconModule } from "@angular/material/icon";
import { RouterLink } from "@angular/router";
import {
  ClipperEntry,
  ClipperEntryKind,
} from "../../../../services/clipboard-history.service";
import { DatePipe } from "@angular/common";
import { openUrl } from "@tauri-apps/plugin-opener";
import { asPlainText, processBytes } from "../../../../utils/text";
import { MatMenuModule, MatMenuTrigger } from "@angular/material/menu";
import { WindowActionsService } from "../../../../services/window-actions.service";
import {
  ClipboardItemDialogComponent,
  ClipboardItemDialogData,
} from "./clipboard-item-dialog.component";
import {
  TagItemDialogComponent,
  TagItemDialogData,
} from "../../../../components/tag-item-dialog/tag-item-dialog.component";
import { TaggedItemKind } from "../../../../services/tags.service";
import { TagSwatchesComponent } from "../../../../components/tag-swatches/tag-swatches.component";

const ITEM_HEIGHT_PX = 120;

@Component({
  selector: "app-clipboard-item",
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: "relative block w-full min-w-0 pb-1",
    "[style.height.px]": "itemHeightPx",
  },
  imports: [
    MatButtonModule,
    MatIconModule,
    RouterLink,
    DatePipe,
    MatMenuModule,
    TagSwatchesComponent,
  ],
  templateUrl: "./clipboard-item.component.html",
  styleUrl: "./clipboard-item.component.scss",
  providers: [],
})
export class ClipboardItemComponent {
  clipperEntry = input.required<ClipperEntry>();
  deleteClicked = output();
  copyClicked = output();
  openClicked = output();
  clickedUrl = signal("");
  ClipperEntryKind = ClipperEntryKind;
  readonly TaggedItemKind = TaggedItemKind;
  menu = viewChild.required<MatMenuTrigger>("linkMenuTrigger");
  contextMenuPosition = { x: "0px", y: "0px" };
  processBytes = processBytes;
  asPlainText = asPlainText;
  openUrl = openUrl;
  readonly itemHeightPx = ITEM_HEIGHT_PX;
  readonly hostElement = inject<ElementRef<HTMLElement>>(ElementRef);
  readonly dialog = inject(MatDialog);
  readonly windowService = inject(WindowActionsService);

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

  showQRCode() {
    this.windowService.hideWindow();
    this.windowService.openQrViewer(this.clickedUrl());
  }

  openExpandedView() {
    this.dialog.open<ClipboardItemDialogComponent, ClipboardItemDialogData>(
      ClipboardItemDialogComponent,
      {
        data: {
          clipperEntry: this.clipperEntry(),
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

  openTagDialog() {
    this.dialog.open<TagItemDialogComponent, TagItemDialogData>(
      TagItemDialogComponent,
      {
        data: {
          itemKind: TaggedItemKind.Clipboard,
          itemId: this.clipperEntry().id,
        },
        autoFocus: false,
      },
    );
  }
}
