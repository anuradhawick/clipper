import { Component } from "@angular/core";
import { NavPaneComponent } from "./components/nav-pane/nav-pane.component";

@Component({
  selector: "app-manager",
  imports: [NavPaneComponent],
  templateUrl: "./manager.component.html",
  styleUrl: "./manager.component.scss",
})
export class ManagerComponent {}
