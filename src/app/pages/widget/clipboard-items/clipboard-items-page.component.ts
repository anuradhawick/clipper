import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  Signal,
} from "@angular/core";
import { toSignal } from "@angular/core/rxjs-interop";
import { ScrollingModule } from "@angular/cdk/scrolling";
import {
  ClipboardHistoryService,
  ClipperEntry,
  ClipperEntryKind,
} from "../../../services/clipboard-history.service";
import { ClipboardItemComponent } from "./clipboard-item/clipboard-item.component";
import { ActivatedRoute } from "@angular/router";
import { asPlainText } from "../../../utils/text";
import {
  ITEM_HEIGHT_PX,
  MAX_BUFFER_PX,
  MIN_BUFFER_PX,
} from "./clipboard-items.constants";

@Component({
  selector: "app-clipboard-items",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [ClipboardItemComponent, ScrollingModule],
  templateUrl: "./clipboard-items-page.component.html",
  styleUrl: "./clipboard-items-page.component.scss",
})
export class ClipboardItemsPageComponent {
  protected readonly chs = inject(ClipboardHistoryService);
  private readonly activatedRoute = inject(ActivatedRoute);
  private readonly asPlainText = asPlainText;
  private readonly queryParamMap = toSignal(this.activatedRoute.queryParamMap, {
    initialValue: this.activatedRoute.snapshot.queryParamMap,
  });
  protected readonly searchQuery = computed(
    () => this.queryParamMap().get("search") ?? "",
  );
  protected readonly clipperEntries: Signal<ClipperEntry[]> = computed(() =>
    this.chs
      .items()
      .filter((entry) => this.matchesSearch(entry, this.searchQuery())),
  );
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;

  protected trackByEntryId(_: number, clipperEntry: ClipperEntry): string {
    return clipperEntry.id;
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
}
