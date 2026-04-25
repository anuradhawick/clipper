import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  input,
  signal,
} from "@angular/core";
import { MatTooltipModule } from "@angular/material/tooltip";
import {
  TaggedItemKind,
  TagEntry,
  TagsService,
} from "../../services/tags.service";

@Component({
  selector: "app-tag-swatches",
  imports: [MatTooltipModule],
  templateUrl: "./tag-swatches.component.html",
  styleUrl: "./tag-swatches.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TagSwatchesComponent {
  readonly itemKind = input.required<TaggedItemKind>();
  readonly itemId = input.required<string>();
  readonly assignedTags = signal<TagEntry[]>([]);
  readonly tagsService = inject(TagsService);
  private loadSequence = 0;

  constructor() {
    effect(() => {
      const itemKind = this.itemKind();
      const itemId = this.itemId();
      // Re-run when assignments or tag metadata change, then query this item.
      this.tagsService.tagItemsVersion();
      this.tagsService.tags();
      void this.loadTags(itemKind, itemId);
    });
  }

  private async loadTags(itemKind: TaggedItemKind, itemId: string) {
    const sequence = ++this.loadSequence;
    const tags = await this.tagsService.readItemTags(itemKind, itemId);
    if (sequence === this.loadSequence) {
      this.assignedTags.set(tags);
    }
  }
}
