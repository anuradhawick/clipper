import { Component, inject } from "@angular/core";
import {
  trigger,
  state,
  style,
  animate,
  transition,
} from "@angular/animations";
import { DropperService } from "../../../../services/dropper.service";

@Component({
  selector: "app-drag-drop-overlay",
  templateUrl: "./drag-drop-overlay.component.html",
  styleUrls: ["./drag-drop-overlay.component.scss"],
  animations: [
    trigger("fadeInOut", [
      state(
        "void",
        style({
          opacity: 0,
        })
      ),
      transition(":enter, :leave", [animate(100)]),
    ]),
  ],
})
export class DragDropOverlayComponent {
  readonly dropperService = inject(DropperService);
}
