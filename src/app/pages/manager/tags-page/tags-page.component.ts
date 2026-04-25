import {
  ChangeDetectionStrategy,
  Component,
  inject,
  signal,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatIconModule } from "@angular/material/icon";
import { MatInputModule } from "@angular/material/input";
import { MatTooltipModule } from "@angular/material/tooltip";
import { ActionConfirmationDialogComponent } from "../../../components/action-confirmation-dialog/action-confirmation-dialog.component";
import { MatDialog } from "@angular/material/dialog";
import {
  TagEditDialogComponent,
  TagEditDialogData,
} from "./tag-edit-dialog/tag-edit-dialog.component";
import { TagItemComponent } from "./tag-item/tag-item.component";
import {
  TAG_COLORS,
  TagEntry,
  TagsService,
} from "../../../services/tags.service";

@Component({
  selector: "app-tags-page",
  imports: [
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatTooltipModule,
    TagItemComponent,
  ],
  templateUrl: "./tags-page.component.html",
  styleUrl: "./tags-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TagsPageComponent {
  readonly tagsService = inject(TagsService);
  readonly dialog = inject(MatDialog);
  readonly tagColors = this.tagsService.tagColors;
  readonly newTag = signal("");
  readonly newKind = signal(TAG_COLORS[0].value);

  setNewKind(kind: string) {
    this.newKind.set(kind);
  }

  async createTag() {
    await this.tagsService.create(this.newTag(), this.newKind());
    this.newTag.set("");
    this.newKind.set(TAG_COLORS[0].value);
  }

  editTag(tag: TagEntry) {
    this.dialog.open<TagEditDialogComponent, TagEditDialogData>(
      TagEditDialogComponent,
      {
        data: { tag },
      },
    );
  }

  deleteTag(tag: TagEntry) {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Delete Tag`,
        message: `Delete "${tag.tag}" and remove it from tagged items?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.tagsService.delete(tag.id);
      }
    });
  }
}
