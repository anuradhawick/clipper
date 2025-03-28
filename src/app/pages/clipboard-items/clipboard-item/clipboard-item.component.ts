import {
  ChangeDetectionStrategy,
  Component,
  input,
  output,
  signal,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { RouterLink } from "@angular/router";
import {
  ClipperEntry,
  ClipperEntryKind,
} from "../../../services/clipboard-history.service";
import { DatePipe } from "@angular/common";
import { openUrl } from "@tauri-apps/plugin-opener";
import { asPlainText, processBytes } from "../../../utils/text";

@Component({
  selector: "app-clipboard-item",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [MatButtonModule, MatIconModule, RouterLink, DatePipe],
  templateUrl: "./clipboard-item.component.html",
  styleUrl: "./clipboard-item.component.scss",
  providers: [],
})
export class ClipboardItemComponent {
  clipperEntry = input.required<ClipperEntry>();
  deleteClicked = output();
  copyClicked = output();
  openClicked = output();
  expanded = signal(false);
  ClipperEntryKind = ClipperEntryKind;
  processBytes = processBytes;
  asPlainText = asPlainText;
  openUrl = openUrl;

  toggleView() {
    this.expanded.update((x) => !x);
  }

  collapse() {
    this.expanded.set(false);
  }

  processImage(image: Array<number>): string {
    const bytes = Uint8Array.from(image);
    const blob = new Blob([bytes], { type: "image" });
    const url = URL.createObjectURL(blob);

    return url;
  }
}
