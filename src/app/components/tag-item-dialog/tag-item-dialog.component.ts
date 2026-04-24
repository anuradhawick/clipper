import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  signal,
} from "@angular/core";
import {
  MAT_DIALOG_DATA,
  MatDialogActions,
  MatDialogClose,
  MatDialogContent,
  MatDialogRef,
  MatDialogTitle,
} from "@angular/material/dialog";
import { MatButtonModule } from "@angular/material/button";
import { MatCheckboxModule } from "@angular/material/checkbox";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatIconModule } from "@angular/material/icon";
import { MatInputModule } from "@angular/material/input";
import { MatTooltipModule } from "@angular/material/tooltip";
import {
  TAG_COLORS,
  TaggedItemKind,
  TagEntry,
  TagsService,
} from "../../services/tags.service";

export interface TagItemDialogData {
  itemKind: TaggedItemKind;
  itemId: string;
}

@Component({
  selector: "app-tag-item-dialog",
  imports: [
    MatButtonModule,
    MatCheckboxModule,
    MatDialogActions,
    MatDialogClose,
    MatDialogContent,
    MatDialogTitle,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatTooltipModule,
  ],
  templateUrl: "./tag-item-dialog.component.html",
  styleUrl: "./tag-item-dialog.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TagItemDialogComponent {
  readonly data = inject<TagItemDialogData>(MAT_DIALOG_DATA);
  readonly dialogRef = inject(MatDialogRef<TagItemDialogComponent>);
  readonly tagsService = inject(TagsService);
  readonly tagColors = this.tagsService.tagColors;
  readonly selectedTagIds = signal<string[]>([]);
  readonly newTag = signal("");
  readonly newKind = signal(TAG_COLORS[0].value);
  readonly saving = signal(false);
  readonly sortedTags = computed(() =>
    [...this.tagsService.tags()].sort((a, b) => a.tag.localeCompare(b.tag)),
  );

  constructor() {
    this.tagsService
      .readItemTags(this.data.itemKind, this.data.itemId)
      .then((tags) => this.selectedTagIds.set(tags.map((tag) => tag.id)));
  }

  isSelected(tagId: string) {
    return this.selectedTagIds().includes(tagId);
  }

  toggleTag(tag: TagEntry) {
    this.selectedTagIds.update((ids) => {
      if (ids.includes(tag.id)) {
        return ids.filter((id) => id !== tag.id);
      }
      return [...ids, tag.id];
    });
  }

  setNewKind(kind: string) {
    this.newKind.set(kind);
  }

  async createTag() {
    const tag = this.newTag().trim();
    if (!tag) {
      return;
    }
    const savedTag = await this.tagsService.create(tag, this.newKind());
    if (savedTag) {
      this.selectedTagIds.update((ids) =>
        ids.includes(savedTag.id) ? ids : [...ids, savedTag.id],
      );
    }
    this.newTag.set("");
    this.newKind.set(TAG_COLORS[0].value);
  }

  async save() {
    this.saving.set(true);
    await this.tagsService.setItemTags(
      this.data.itemKind,
      this.data.itemId,
      this.selectedTagIds(),
    );
    this.dialogRef.close(true);
  }
}
