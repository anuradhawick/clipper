import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
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
import { NoteItem } from "../../../../services/notes.service";
import { DatePipe } from "@angular/common";
import { asPlainText, processText } from "../../../../utils/text";
import { openUrl } from "@tauri-apps/plugin-opener";
import { MatMenuModule, MatMenuTrigger } from "@angular/material/menu";
import { WindowActionsService } from "../../../../services/window-actions.service";
import { MatDialog } from "@angular/material/dialog";
import {
  NoteItemDialogComponent,
  NoteItemDialogData,
} from "./note-item-dialog.component";
import {
  TagItemDialogComponent,
  TagItemDialogData,
} from "../../../../components/tag-item-dialog/tag-item-dialog.component";
import { TaggedItemKind } from "../../../../services/tags.service";

const ITEM_HEIGHT_PX = 120;

@Component({
  selector: "app-note-item",
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: "block w-full min-w-0 pb-1",
    "[style.height.px]": "itemHeightPx",
  },
  imports: [MatButtonModule, MatIconModule, DatePipe, MatMenuModule],
  templateUrl: "./note-item.component.html",
  styleUrl: "./note-item.component.scss",
  providers: [],
})
export class NoteItemComponent {
  note = input.required<NoteItem>();
  deleteClicked = output();
  copyClicked = output();
  clickedUrl = signal("");
  contentUpdated = output<string>();
  menu = viewChild.required<MatMenuTrigger>(MatMenuTrigger);
  editable = signal(false);
  editor = viewChild<ElementRef>("editor");
  contextMenuPosition = { x: "0px", y: "0px" };
  dateFmt: any;
  processText = processText;
  asPlainText = asPlainText;
  openUrl = openUrl;
  readonly itemHeightPx = ITEM_HEIGHT_PX;
  readonly changeDetectorRef = inject(ChangeDetectorRef);
  readonly windowService = inject(WindowActionsService);
  readonly dialog = inject(MatDialog);

  uneditable() {
    this.editable.set(false);
  }

  toggleEditable() {
    this.editable.update((x) => !x);
    if (this.editable()) {
      this.changeDetectorRef.detectChanges();
      this.editor()!.nativeElement.focus();
    }
  }

  updateNote() {
    const note = this.editor()!.nativeElement.innerText || "";
    this.editor()!.nativeElement.innerHTML = "";
    this.editor()!.nativeElement.innerText = "";
    this.uneditable();
    this.contentUpdated.emit(note);
  }

  onLinkRightClick(event: MouseEvent, url: string) {
    this.contextMenuPosition.x = event.clientX + "px";
    this.contextMenuPosition.y = event.clientY + "px";
    this.clickedUrl.set(url);
    this.menu().openMenu();
  }

  showQRCode() {
    this.windowService.hideWindow();
    this.windowService.openQrViewer(this.clickedUrl());
  }

  openExpandedView() {
    this.dialog.open<NoteItemDialogComponent, NoteItemDialogData>(
      NoteItemDialogComponent,
      {
        data: {
          note: this.note(),
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
          itemKind: TaggedItemKind.Note,
          itemId: this.note().id,
        },
        autoFocus: false,
      },
    );
  }
}
