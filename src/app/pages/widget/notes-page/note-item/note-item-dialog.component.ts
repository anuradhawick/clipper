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
import { NoteItem } from "../../../../services/notes.service";
import { processText } from "../../../../utils/text";

export interface NoteItemDialogData {
  note: NoteItem;
}

@Component({
  selector: "app-note-item-dialog",
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  imports: [DatePipe, MatButtonModule, MatDialogClose, MatIconModule],
  templateUrl: "./note-item-dialog.component.html",
  styleUrl: "./note-item-dialog.component.scss",
})
export class NoteItemDialogComponent {
  readonly data = inject<NoteItemDialogData>(MAT_DIALOG_DATA);
  readonly processText = processText;
  readonly openUrl = openUrl;
}
