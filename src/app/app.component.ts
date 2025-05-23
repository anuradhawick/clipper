import { Component, HostListener } from "@angular/core";
import { CommonModule } from "@angular/common";
import { RouterOutlet } from "@angular/router";

@Component({
  selector: "app-root",
  imports: [CommonModule, RouterOutlet],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.scss",
  providers: [RouterOutlet],
})
export class AppComponent {
  @HostListener("document:contextmenu", ["$event"])
  onRightClick(event: MouseEvent): void {
    event.preventDefault();
  }
}
