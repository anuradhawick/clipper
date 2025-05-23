import {
  AfterViewInit,
  Component,
  ElementRef,
  inject,
  OnDestroy,
  OnInit,
  signal,
  viewChild,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { ActivatedRoute, ParamMap, Router, RouterLink } from "@angular/router";
import { Subscription } from "rxjs";
import { NotesService } from "../../../../services/notes.service";

@Component({
  selector: "app-new-note",
  imports: [MatIconModule, MatButtonModule, RouterLink],
  templateUrl: "./new-note.component.html",
  styleUrl: "./new-note.component.scss",
})
export class NewNoteComponent implements OnInit, OnDestroy, AfterViewInit {
  paramsSub: Subscription | null = null;
  entry = signal("");
  newEntry = signal("");
  editor = viewChild.required<ElementRef>("editor");
  readonly route = inject(ActivatedRoute);
  readonly router = inject(Router);
  readonly notesService = inject(NotesService);

  ngOnInit(): void {
    this.paramsSub = this.route.paramMap.subscribe((params: ParamMap) => {
      this.entry.set(params.get("entry") || "");
      this.newEntry.set(params.get("entry") || "");
    });
  }

  ngAfterViewInit(): void {
    this.editor().nativeElement.focus();
  }

  ngOnDestroy(): void {
    if (this.paramsSub) {
      this.paramsSub.unsubscribe();
    }
  }

  async save() {
    // do not allow saving if the trimmed length is nil
    if (this.newEntry().trim().length === 0) {
      return;
    }
    // create note and redirect
    await this.notesService.create(this.newEntry());
    this.router.navigate(["/clipper", "notes"]);
  }

  change(event: Event) {
    // set new entry with new text even with spaces
    this.newEntry.set((event.target as HTMLDivElement).innerText || "");
  }
}
