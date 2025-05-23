import { Component, inject } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { NotesService } from "../../../services/notes.service";
import { NoteItemComponent } from "./note-item/note-item.component";
import { RouterOutlet } from "@angular/router";

@Component({
  selector: "app-notes-page",
  imports: [MatButtonModule, MatIconModule, NoteItemComponent, RouterOutlet],
  templateUrl: "./notes-page.component.html",
  styleUrl: "./notes-page.component.scss",
})
export class NotesPageComponent {
  readonly notesService = inject(NotesService);

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
}
