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
import {
  ClipperEntry,
  ClipperEntryKind,
} from "../../../../services/clipboard-history.service";
import { processBytes } from "../../../../utils/text";

export interface ClipboardItemDialogData {
  clipperEntry: ClipperEntry;
}

@Component({
  selector: "app-clipboard-item-dialog",
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  imports: [DatePipe, MatButtonModule, MatDialogClose, MatIconModule],
  templateUrl: "./clipboard-item-dialog.component.html",
  styleUrl: "./clipboard-item-dialog.component.scss",
})
export class ClipboardItemDialogComponent {
  readonly data = inject<ClipboardItemDialogData>(MAT_DIALOG_DATA);
  readonly ClipperEntryKind = ClipperEntryKind;
  readonly processBytes = processBytes;
  readonly openUrl = openUrl;

  processImage(image: Array<number>): string {
    const bytes = Uint8Array.from(image);
    const blob = new Blob([bytes], { type: "image" });

    return URL.createObjectURL(blob);
  }
}
