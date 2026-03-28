import { ChangeDetectionStrategy, Component } from "@angular/core";
import { NavPaneComponent } from "./components/nav-pane/nav-pane.component";
import { RouterOutlet } from "@angular/router";

@Component({
  selector: "app-manager",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [NavPaneComponent, RouterOutlet],
  templateUrl: "./manager.component.html",
  styleUrl: "./manager.component.scss",
})
export class ManagerComponent {}
