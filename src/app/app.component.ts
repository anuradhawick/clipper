import { Component } from "@angular/core";
import { CommonModule } from "@angular/common";
import { RouterOutlet } from "@angular/router";
import { invoke } from "@tauri-apps/api/tauri";
import { ClipboardItemsPageComponent } from "./pages/clipboard-items/clipboard-items-page.component";
import { IconsRegistrar } from "./app.icons";
import { ClipboardHistoryService } from "./services/clipboard-history.service";
import { NavBarComponent } from "./components/nav-bar/nav-bar.component";
import { ThemeService } from "./services/theme.service";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [
    CommonModule,
    RouterOutlet,
    ClipboardItemsPageComponent,
    NavBarComponent,
  ],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.scss",
  providers: [IconsRegistrar, RouterOutlet],
})
export class AppComponent {
  greetingMessage = "";

  constructor(
    private icons: IconsRegistrar,
    private ts: ThemeService,
    protected chs: ClipboardHistoryService
  ) {
    this.icons.registerIcons();
  }

  greet(event: SubmitEvent, name: string): void {
    event.preventDefault();

    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    invoke<string>("greet", { name }).then((text) => {
      this.greetingMessage = text;
    });
  }
}
