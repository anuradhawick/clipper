import { ChangeDetectionStrategy, Component } from "@angular/core";
import { RouterOutlet } from "@angular/router";
import { NavBarComponent } from "./components/nav-bar/nav-bar.component";
import { DragDropOverlayComponent } from "./components/drag-drop-overlay/drag-drop-overlay.component";

const MAIN_WINDOW_WIDTH = 800;
const MAIN_WINDOW_HEIGHT = 400;

@Component({
  selector: "app-widget",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [RouterOutlet, NavBarComponent, DragDropOverlayComponent],
  templateUrl: "./widget.component.html",
  styleUrl: "./widget.component.scss",
  providers: [RouterOutlet],
})
export class WidgetComponent {
  protected readonly windowSize = {
    width: MAIN_WINDOW_WIDTH,
    height: MAIN_WINDOW_HEIGHT,
  } as const;
}
