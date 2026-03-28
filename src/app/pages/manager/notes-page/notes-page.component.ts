import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import { toSignal } from "@angular/core/rxjs-interop";
import { ScrollingModule } from "@angular/cdk/scrolling";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { NoteItem, NotesService } from "../../../services/notes.service";
import { NoteItemComponent } from "./note-item/note-item.component";
import {
  NavigationEnd,
  Router,
  RouterLink,
  RouterOutlet,
} from "@angular/router";
import { MatDialog } from "@angular/material/dialog";
import { ActionConfirmationDialogComponent } from "../../../components/action-confirmation-dialog/action-confirmation-dialog.component";
import { MatTooltipModule } from "@angular/material/tooltip";
import { filter, map, startWith } from "rxjs";

const ITEM_HEIGHT_PX = 120;
const MIN_BUFFER_PX = 240;
const MAX_BUFFER_PX = 480;

@Component({
  selector: "app-notes-page",
  imports: [
    MatButtonModule,
    MatIconModule,
    MatTooltipModule,
    NoteItemComponent,
    RouterLink,
    RouterOutlet,
    ScrollingModule,
  ],
  templateUrl: "./notes-page.component.html",
  styleUrl: "./notes-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NotesPageComponent {
  readonly notesService = inject(NotesService);
  readonly dialog = inject(MatDialog);
  readonly router = inject(Router);
  protected readonly itemHeightPx = ITEM_HEIGHT_PX;
  protected readonly minBufferPx = MIN_BUFFER_PX;
  protected readonly maxBufferPx = MAX_BUFFER_PX;
  protected readonly isCreatingNote = toSignal(
    this.router.events.pipe(
      filter((event) => event instanceof NavigationEnd),
      startWith(null),
      map(() => this.router.url.includes("/notes/new")),
    ),
    { initialValue: this.router.url.includes("/notes/new") },
  );

  protected trackByNoteId(_: number, note: NoteItem): string {
    return note.id;
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
