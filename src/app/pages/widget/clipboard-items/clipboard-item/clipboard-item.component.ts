import {
  ChangeDetectionStrategy,
  Component,
  inject,
  input,
  output,
  signal,
  viewChild,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { RouterLink } from "@angular/router";
import {
  ClipperEntry,
  ClipperEntryKind,
} from "../../../../services/clipboard-history.service";
import { DatePipe } from "@angular/common";
import { openUrl } from "@tauri-apps/plugin-opener";
import { asPlainText, processBytes } from "../../../../utils/text";
import { MatMenuModule, MatMenuTrigger } from "@angular/material/menu";
import { WindowActionsService } from "../../../../services/window-actions.service";

@Component({
  selector: "app-clipboard-item",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    MatButtonModule,
    MatIconModule,
    RouterLink,
    DatePipe,
    MatMenuModule,
  ],
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
  clickedUrl = signal("");
  ClipperEntryKind = ClipperEntryKind;
  menu = viewChild.required<MatMenuTrigger>(MatMenuTrigger);
  contextMenuPosition = { x: "0px", y: "0px" };
  processBytes = processBytes;
  asPlainText = asPlainText;
  openUrl = openUrl;
  readonly windowService = inject(WindowActionsService);

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

  onLinkRightClick(event: MouseEvent, url: string) {
    this.contextMenuPosition.x = event.clientX + "px";
    this.contextMenuPosition.y = event.clientY + "px";
    this.clickedUrl.set(url);
    this.menu().openMenu();
  }

  showQRCode() {
    this.windowService.hideWindow();
    this.windowService.openQrViewer(this.clickedUrl());
  }
}
