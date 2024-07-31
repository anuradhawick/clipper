import {
  AfterViewInit,
  ChangeDetectorRef,
  Component,
  ElementRef,
  input,
  output,
  signal,
  viewChild,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { NoteItem } from "../../../services/notes.service";
import { DatePipe } from "@angular/common";

@Component({
  selector: "app-note-item",
  standalone: true,
  imports: [MatButtonModule, MatIconModule, DatePipe],
  templateUrl: "./note-item.component.html",
  styleUrl: "./note-item.component.scss",
  providers: [],
})
export class NoteItemComponent {
  note = input.required<NoteItem>();
  deleteClicked = output();
  copyClicked = output();
  contentUpdated = output<string>();
  expanded = signal(false);
  editable = signal(false);
  editor = viewChild.required<ElementRef>("editor");
  dateFmt: any;

  constructor(private cd: ChangeDetectorRef) {}

  toggleView() {
    this.expanded.update((x) => !x);
  }

  collapse() {
    this.expanded.set(false);
  }

  uneditable() {
    this.editable.set(false);
  }

  toggleEditable() {
    this.editable.update((x) => !x);
    if (this.editable()) {
      this.expanded.set(true);
      this.cd.detectChanges();
      this.editor().nativeElement.focus();
    }
  }
}
