import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  Signal,
} from "@angular/core";
import { ScrollingModule } from "@angular/cdk/scrolling";
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

const ITEM_HEIGHT_PX = 120;
const MIN_BUFFER_PX = 240;
const MAX_BUFFER_PX = 480;

@Component({
  selector: "app-clipboard-page",
  imports: [
    ClipboardItemComponent,
    ScrollingModule,
    MatButtonModule,
    MatIconModule,
    MatTooltipModule,
  ],
  templateUrl: "./clipboard-page.component.html",
  styleUrl: "./clipboard-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ClipboardPageComponent {
  protected readonly chs = inject(ClipboardHistoryService);
  readonly dialog = inject(MatDialog);
  protected readonly clipperEntries: Signal<ClipperEntry[]> = computed(() =>
    this.chs.items(),
  );
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;

  protected trackByEntryId(_: number, clipperEntry: ClipperEntry): string {
    return clipperEntry.id;
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
