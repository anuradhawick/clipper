import {
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

@Component({
  selector: "app-note-item",
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
  expanded = signal(false);
  editable = signal(false);
  editor = viewChild<ElementRef>("editor");
  contextMenuPosition = { x: "0px", y: "0px" };
  dateFmt: any;
  processText = processText;
  asPlainText = asPlainText;
  openUrl = openUrl;
  readonly changeDetectorRef = inject(ChangeDetectorRef);
  readonly windowService = inject(WindowActionsService);

  toggleView() {
    this.expanded.update((x) => !x);
  }

  collapse() {
    this.expanded.set(false);
  }

  uneditable() {
    this.editable.set(false);
  }

  toggleEditable() {
    this.editable.update((x) => !x);
    if (this.editable()) {
      this.expanded.set(true);
      this.changeDetectorRef.detectChanges();
      this.editor()!.nativeElement.focus();
    }
  }

  updateNote() {
    const note = this.editor()!.nativeElement.innerText || "";
    this.editor()!.nativeElement.innerHTML = "";
    this.editor()!.nativeElement.innerText = "";
    this.uneditable();
    this.collapse();
    this.contentUpdated.emit(note);
  }

  onLinkRightClick(event: MouseEvent, url: string) {
    this.contextMenuPosition.x = event.clientX + "px";
    this.contextMenuPosition.y = event.clientY + "px";
    event.preventDefault();
    this.menu().openMenu();
    this.clickedUrl.set(url);
  }

  showQRCode() {
    this.windowService.hideWindow();
    this.windowService.openQrViewer(this.clickedUrl());
  }
}
