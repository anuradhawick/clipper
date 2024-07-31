import { Component, input, output, signal } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { RouterLink } from "@angular/router";
import { ClipperEntry } from "../../../services/clipboard-history.service";
import { DatePipe } from "@angular/common";

@Component({
  selector: "app-clipboard-item",
  standalone: true,
  imports: [MatButtonModule, MatIconModule, RouterLink, DatePipe],
  templateUrl: "./clipboard-item.component.html",
  styleUrl: "./clipboard-item.component.scss",
  providers: [],
})
export class ClipboardItemComponent {
  clipperEntry = input.required<ClipperEntry>();
  deleteClicked = output();
  copyClicked = output();
  expanded = signal(false);

  toggleView() {
    this.expanded.update((x) => !x);
  }

  collapse() {
    this.expanded.set(false);
  }
}
