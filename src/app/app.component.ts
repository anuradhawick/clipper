import { Component } from "@angular/core";
import { CommonModule } from "@angular/common";
import { RouterOutlet } from "@angular/router";
import { invoke } from "@tauri-apps/api/core";
import { ClipboardItemsPageComponent } from "./pages/clipboard-items/clipboard-items-page.component";
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
  providers: [RouterOutlet],
})
export class AppComponent {
  greetingMessage = "";

  constructor(
    private ts: ThemeService,
    protected chs: ClipboardHistoryService
  ) {}

  greet(event: SubmitEvent, name: string): void {
    event.preventDefault();

    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    invoke<string>("greet", { name }).then((text) => {
      this.greetingMessage = text;
    });
  }
}
