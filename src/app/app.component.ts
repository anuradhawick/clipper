import { Component } from "@angular/core";
import { CommonModule } from "@angular/common";
import { RouterOutlet } from "@angular/router";
import { ClipboardItemsPageComponent } from "./pages/clipboard-items/clipboard-items-page.component";
import { NavBarComponent } from "./components/nav-bar/nav-bar.component";

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
export class AppComponent {}
