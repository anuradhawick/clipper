import { Component } from "@angular/core";

import { RouterOutlet } from "@angular/router";
import { NavBarComponent } from "./components/nav-bar/nav-bar.component";
import { DragDropOverlayComponent } from "./components/drag-drop-overlay/drag-drop-overlay.component";

@Component({
  selector: "app-widget",
  imports: [
    RouterOutlet,
    NavBarComponent,
    DragDropOverlayComponent
],
  templateUrl: "./widget.component.html",
  styleUrl: "./widget.component.scss",
  providers: [RouterOutlet],
})
export class WidgetComponent {}
