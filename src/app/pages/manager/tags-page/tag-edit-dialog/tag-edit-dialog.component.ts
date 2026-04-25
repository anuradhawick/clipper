import {
  ChangeDetectionStrategy,
  Component,
  inject,
  signal,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import {
  MAT_DIALOG_DATA,
  MatDialogActions,
  MatDialogClose,
  MatDialogContent,
  MatDialogRef,
  MatDialogTitle,
} from "@angular/material/dialog";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatIconModule } from "@angular/material/icon";
import { MatInputModule } from "@angular/material/input";
import { MatTooltipModule } from "@angular/material/tooltip";
import {
  AbstractControl,
  NonNullableFormBuilder,
  ReactiveFormsModule,
  ValidationErrors,
  Validators,
} from "@angular/forms";
import {
  TAG_COLORS,
  TagEntry,
  TagsService,
} from "../../../../services/tags.service";

export interface TagEditDialogData {
  tag: TagEntry;
}

@Component({
  selector: "app-tag-edit-dialog",
  imports: [
    MatButtonModule,
    MatDialogActions,
    MatDialogClose,
    MatDialogContent,
    MatDialogTitle,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    ReactiveFormsModule,
    MatTooltipModule,
  ],
  templateUrl: "./tag-edit-dialog.component.html",
  styleUrl: "./tag-edit-dialog.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TagEditDialogComponent {
  readonly data = inject<TagEditDialogData>(MAT_DIALOG_DATA);
  readonly dialogRef = inject(MatDialogRef<TagEditDialogComponent>);
  readonly tagsService = inject(TagsService);
  private readonly formBuilder = inject(NonNullableFormBuilder);
  protected readonly tagColors = TAG_COLORS;
  protected readonly form = this.formBuilder.group({
    tag: [this.data.tag.tag, [Validators.required, this.nonBlankTagValidator]],
    kind: [this.data.tag.kind || TAG_COLORS[0].value, Validators.required],
  });
  protected readonly tagControl = this.form.controls.tag;
  protected readonly saving = signal(false);

  protected setEditKind(kind: string) {
    this.form.controls.kind.setValue(kind);
  }

  protected async saveTag() {
    if (this.form.invalid || this.saving()) {
      this.form.markAllAsTouched();
      return;
    }

    const value = this.form.getRawValue();
    this.saving.set(true);
    await this.tagsService.update(
      this.data.tag.id,
      value.tag.trim(),
      value.kind,
    );
    this.dialogRef.close(true);
  }

  private nonBlankTagValidator(
    control: AbstractControl<string>,
  ): ValidationErrors | null {
    const value = control.value;
    if (!value) {
      return null;
    }

    return value.trim().length ? null : { nonBlankTag: true };
  }
}
