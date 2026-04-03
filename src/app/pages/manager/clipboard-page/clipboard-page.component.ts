import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  inject,
  signal,
  Signal,
  viewChild,
  ViewChild,
} from "@angular/core";
import { ScrollingModule } from "@angular/cdk/scrolling";
import {
  ClipboardHistoryService,
  ClipperEntryKind,
  ClipperEntry,
} from "../../../services/clipboard-history.service";
import { ClipboardItemComponent } from "./clipboard-item/clipboard-item.component";
import { MatIconModule } from "@angular/material/icon";
import { MatButtonModule } from "@angular/material/button";
import { MatTooltipModule } from "@angular/material/tooltip";
import { MatDialog } from "@angular/material/dialog";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { ActionConfirmationDialogComponent } from "../../../components/action-confirmation-dialog/action-confirmation-dialog.component";
import { asPlainText } from "../../../utils/text";

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
    MatFormFieldModule,
    MatInputModule,
  ],
  templateUrl: "./clipboard-page.component.html",
  styleUrl: "./clipboard-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ClipboardPageComponent {
  protected readonly chs = inject(ClipboardHistoryService);
  readonly dialog = inject(MatDialog);
  private readonly asPlainText = asPlainText;
  private searchInputRef =
    viewChild<ElementRef<HTMLInputElement>>("searchInput");
  protected readonly clipperEntries: Signal<ClipperEntry[]> = computed(() =>
    this.chs
      .items()
      .filter((entry) => this.matchesSearch(entry, this.searchQuery())),
  );
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;
  protected showSearch = signal(false);
  protected searchQuery = signal("");

  protected trackByEntryId(_: number, clipperEntry: ClipperEntry): string {
    return clipperEntry.id;
  }

  protected toggleSearch(): void {
    const shouldShow = !this.showSearch();
    this.showSearch.set(shouldShow);

    if (shouldShow) {
      setTimeout(() => this.searchInputRef()?.nativeElement.focus());
    } else {
      this.searchQuery.set("");
    }
  }

  protected clearSearch(searchInput: HTMLInputElement) {
    searchInput.value = "";
    this.searchQuery.set("");
  }

  private matchesSearch(entry: ClipperEntry, query: string): boolean {
    if (!query) {
      return true;
    }

    if (entry.kind !== ClipperEntryKind.Text) {
      return false;
    }

    return this.asPlainText(entry.entry)
      .toLowerCase()
      .includes(query.toLowerCase());
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
