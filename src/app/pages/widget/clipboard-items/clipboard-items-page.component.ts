import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  Signal,
  signal,
} from "@angular/core";
import { ScrollingModule } from "@angular/cdk/scrolling";
import {
  ClipboardHistoryService,
  ClipperEntry,
  ClipperEntryKind,
} from "../../../services/clipboard-history.service";
import { ClipboardItemComponent } from "./clipboard-item/clipboard-item.component";
import {
  ITEM_HEIGHT_PX,
  MAX_BUFFER_PX,
  MIN_BUFFER_PX,
} from "./clipboard-items.constants";
import { FormsModule } from "@angular/forms";
import { MatIconModule } from "@angular/material/icon";
import { MatButtonModule } from "@angular/material/button";

@Component({
  selector: "app-clipboard-items",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    ClipboardItemComponent,
    ScrollingModule,
    FormsModule,
    MatIconModule,
    MatButtonModule,
  ],
  templateUrl: "./clipboard-items-page.component.html",
  styleUrl: "./clipboard-items-page.component.scss",
})
export class ClipboardItemsPageComponent {
  protected readonly chs = inject(ClipboardHistoryService);
  protected readonly searchQuery = signal("");
  protected readonly filteredEntries: Signal<ClipperEntry[]> = computed(() => {
    const query = this.searchQuery().toLowerCase().trim();
    if (!query) return this.chs.items();
    return this.chs.items().filter((entry) => {
      if (entry.kind === ClipperEntryKind.Text) {
        const text = new TextDecoder()
          .decode(Uint8Array.from(entry.entry))
          .toLowerCase();
        return text.includes(query);
      }
      return false;
    });
  });
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;

  protected trackByEntryId(_: number, clipperEntry: ClipperEntry): string {
    return clipperEntry.id;
  }

  protected clearSearch(): void {
    this.searchQuery.set("");
  }
}
