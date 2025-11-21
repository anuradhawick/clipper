import { Component } from "@angular/core";
import { NavPaneComponent } from "./components/nav-pane/nav-pane.component";
import { RouterOutlet } from "@angular/router";

@Component({
  selector: "app-manager",
  imports: [NavPaneComponent, RouterOutlet],
  templateUrl: "./manager.component.html",
  styleUrl: "./manager.component.scss",
})
export class ManagerComponent {}
