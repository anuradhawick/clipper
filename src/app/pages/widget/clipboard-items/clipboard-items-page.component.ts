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
  protected readonly clipperEntries: Signal<ClipperEntry[]> = computed(() =>
    this.chs.items(),
  );
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;

  protected trackByEntryId(_: number, clipperEntry: ClipperEntry): string {
    return clipperEntry.id;
  }
}
