import { Component, inject } from "@angular/core";
import { MatRippleModule } from "@angular/material/core";
import { ThemeService } from "../../../../services/theme.service";
import { RouterLink } from "@angular/router";
import { MatBadgeModule } from "@angular/material/badge";
import { ClipboardHistoryService } from "../../../../services/clipboard-history.service";

@Component({
  selector: "app-nav-pane",
  imports: [MatRippleModule, RouterLink, MatBadgeModule],
  templateUrl: "./nav-pane.component.html",
  styleUrl: "./nav-pane.component.scss",
})
export class NavPaneComponent {
  themeService = inject(ThemeService);
  clipboardHistoryService = inject(ClipboardHistoryService);
}
