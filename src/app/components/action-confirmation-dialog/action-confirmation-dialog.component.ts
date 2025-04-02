import { Component, inject } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import {
  MAT_DIALOG_DATA,
  MatDialogActions,
  MatDialogClose,
  MatDialogContent,
  MatDialogTitle,
} from "@angular/material/dialog";

export interface ConfirmationDialogData {
  title: string;
  message: string;
}

@Component({
  selector: "app-action-confirmation-dialog",
  imports: [
    MatDialogTitle,
    MatDialogContent,
    MatDialogActions,
    MatDialogClose,
    MatButtonModule,
  ],
  templateUrl: "./action-confirmation-dialog.component.html",
  styleUrl: "./action-confirmation-dialog.component.scss",
})
export class ActionConfirmationDialogComponent {
  readonly data = inject<ConfirmationDialogData>(MAT_DIALOG_DATA);
}
