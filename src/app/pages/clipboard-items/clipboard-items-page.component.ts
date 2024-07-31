import { Component, computed, Signal, signal } from "@angular/core";
import {
  ClipboardHistoryService,
  ClipperEntry,
  ClipperEntryKind,
} from "../../services/clipboard-history.service";
import { ClipboardItemComponent } from "./clipboard-item/clipboard-item.component";
import { MatIconModule } from "@angular/material/icon";
import { MatButtonModule } from "@angular/material/button";

@Component({
  selector: "app-clipboard-items",
  standalone: true,
  imports: [
    ClipboardItemsPageComponent,
    ClipboardItemComponent,
    MatButtonModule,
    MatIconModule,
  ],
  templateUrl: "./clipboard-items-page.component.html",
  styleUrl: "./clipboard-items-page.component.scss",
})
export class ClipboardItemsPageComponent {
  clipperEntries: Signal<ClipperEntry[]>;

  constructor(protected chs: ClipboardHistoryService) {
    this.clipperEntries = computed(() => chs.items());
    // this.clipperEntries = computed(() => [
    //   {
    //     id: "1",
    //     kind: ClipperEntryKind.Text,
    //     entry: "This is an inline test",
    //   },
    //   {
    //     id: "2",
    //     kind: ClipperEntryKind.Text,
    //     entry:
    //       "This is an inline test for a very very very long one that might actually have some very ugly overflow",
    //   },
    //   {
    //     id: "3",
    //     kind: ClipperEntryKind.Text,
    //     entry: `This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>`,
    //   },
    //   {
    //     id: "4",
    //     kind: ClipperEntryKind.Text,
    //     entry: `This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>`,
    //   },
    //   {
    //     id: "5",
    //     kind: ClipperEntryKind.Text,
    //     entry: `This is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nThis is a multi line test\nwith many many lines\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>\nmay be too long for the <pre></pre>`,
    //   },
    // ]);
  }
}
