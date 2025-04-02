import { Component } from "@angular/core";
import { CommonModule } from "@angular/common";
import { RouterOutlet } from "@angular/router";
import { NavBarComponent } from "./components/nav-bar/nav-bar.component";
import { DragDropOverlayComponent } from "./components/drag-drop-overlay/drag-drop-overlay.component";

@Component({
  selector: "app-root",
  imports: [
    CommonModule,
    RouterOutlet,
    NavBarComponent,
    DragDropOverlayComponent,
  ],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.scss",
  providers: [RouterOutlet],
})
export class AppComponent {}
