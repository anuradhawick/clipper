import { Component, inject } from "@angular/core";
import { MatRippleModule } from "@angular/material/core";
import { ThemeService } from "../../../../services/theme.service";

@Component({
  selector: "app-nav-pane",
  imports: [MatRippleModule],
  templateUrl: "./nav-pane.component.html",
  styleUrl: "./nav-pane.component.scss",
})
export class NavPaneComponent {
  themeService = inject(ThemeService);
}
