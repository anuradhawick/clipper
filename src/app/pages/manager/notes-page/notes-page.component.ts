import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { NotesService } from "../../../services/notes.service";
import { NoteItemComponent } from "./note-item/note-item.component";
import { RouterLink, RouterOutlet } from "@angular/router";
import { MatDialog } from "@angular/material/dialog";
import { ActionConfirmationDialogComponent } from "../../../components/action-confirmation-dialog/action-confirmation-dialog.component";
import { MatTooltipModule } from "@angular/material/tooltip";

@Component({
  selector: "app-notes-page",
  imports: [
    MatButtonModule,
    MatIconModule,
    MatTooltipModule,
    NoteItemComponent,
    RouterLink,
    RouterOutlet,
  ],
  templateUrl: "./notes-page.component.html",
  styleUrl: "./notes-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NotesPageComponent {
  readonly notesService = inject(NotesService);
  readonly dialog = inject(MatDialog);

  constructor() {
    // this.notes = computed(() => [
    //   { id: "1", entry: "This is a note. This is an inline test" },
    //   {
    //     id: "2",
    //     entry:
    //       "This is a note. This is an inline test for a very very very long one that might actually have some very ugly overflow",
    //   },
    //   {
    //     id: "3",
    //     entry: `This is a note. This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>`,
    //   },
    //   {
    //     id: "4",
    //     entry: `This is a note. This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>`,
    //   },
    //   {
    //     id: "5",
    //     entry: `This is a note. This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>`,
    //   },
    // ]);
  }

  deleteNotes() {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Clear Notes`,
        message: `Are you sure you want to clear all notes?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.notesService.deleteAll();
      }
    });
  }
}
