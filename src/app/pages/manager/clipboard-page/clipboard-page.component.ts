import { Component, computed, inject, Signal } from "@angular/core";
import {
  ClipboardHistoryService,
  ClipperEntry,
} from "../../../services/clipboard-history.service";
import { ClipboardItemComponent } from "./clipboard-item/clipboard-item.component";
import { MatIconModule } from "@angular/material/icon";
import { MatButtonModule } from "@angular/material/button";
import { MatTooltipModule } from "@angular/material/tooltip";
import { MatDialog } from "@angular/material/dialog";
import { ActionConfirmationDialogComponent } from "../../../components/action-confirmation-dialog/action-confirmation-dialog.component";

@Component({
  selector: "app-clipboard-page",
  imports: [
    ClipboardItemComponent,
    MatButtonModule,
    MatIconModule,
    MatTooltipModule,
  ],
  templateUrl: "./clipboard-page.component.html",
  styleUrl: "./clipboard-page.component.scss",
})
export class ClipboardPageComponent {
  clipperEntries: Signal<ClipperEntry[]>;
  readonly dialog = inject(MatDialog);

  constructor(protected chs: ClipboardHistoryService) {
    this.clipperEntries = computed(() => chs.items());
    // this.clipperEntries = computed(() => [
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

  clearClipboardHistory() {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Clear Clipboard History`,
        message: `Are you sure you want to clear all clipboard entries?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.chs.clear();
      }
    });
  }
}
