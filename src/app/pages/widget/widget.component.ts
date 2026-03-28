import { ChangeDetectionStrategy, Component } from "@angular/core";
import { RouterOutlet } from "@angular/router";
import tauriConfig from "../../../../src-tauri/tauri.conf.json";
import { NavBarComponent } from "./components/nav-bar/nav-bar.component";
import { DragDropOverlayComponent } from "./components/drag-drop-overlay/drag-drop-overlay.component";

interface TauriWindowConfig {
  label?: string;
  width?: number;
  height?: number;
}

interface TauriAppConfig {
  windows?: TauriWindowConfig[];
}

interface TauriConfig {
  app?: TauriAppConfig;
}

const mainWindowConfig = (tauriConfig as TauriConfig).app?.windows?.find(
  (windowConfig) => windowConfig.label === "main",
);

if (!mainWindowConfig?.width || !mainWindowConfig?.height) {
  throw new Error("Unable to resolve the main Tauri window size.");
}

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
    width: mainWindowConfig.width,
    height: mainWindowConfig.height,
  } as const;
}
