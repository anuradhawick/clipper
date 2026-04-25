import {
  ChangeDetectionStrategy,
  Component,
  computed,
  input,
  output,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { MatTooltipModule } from "@angular/material/tooltip";
import { TAG_COLORS, TagEntry } from "../../../../services/tags.service";

@Component({
  selector: "app-tag-item",
  imports: [MatButtonModule, MatIconModule, MatTooltipModule],
  templateUrl: "./tag-item.component.html",
  styleUrl: "./tag-item.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TagItemComponent {
  readonly tag = input.required<TagEntry>();
  readonly editClicked = output<TagEntry>();
  readonly deleteClicked = output<TagEntry>();

  protected readonly colorLabel = computed(() => {
    const kind = this.tag().kind;
    return TAG_COLORS.find((color) => color.value === kind)?.label ?? kind;
  });

  protected editTag() {
    this.editClicked.emit(this.tag());
  }

  protected deleteTag() {
    this.deleteClicked.emit(this.tag());
  }
}
