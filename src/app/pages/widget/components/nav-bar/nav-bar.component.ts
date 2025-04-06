import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  inject,
  signal,
  viewChild,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { ClipboardHistoryService } from "../../../../services/clipboard-history.service";
import { WindowActionsService } from "../../../../services/window-actions.service";
import { Router, RouterLink } from "@angular/router";
import { Subscription } from "rxjs";
import { MatMenu, MatMenuModule, MatMenuTrigger } from "@angular/material/menu";
import { MatDialog } from "@angular/material/dialog";
import { ActionConfirmationDialogComponent } from "../../../../components/action-confirmation-dialog/action-confirmation-dialog.component";
import { MatBadgeModule } from "@angular/material/badge";
import { DropperService } from "../../../../services/dropper.service";

@Component({
  selector: "app-nav-bar",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    MatButtonModule,
    MatIconModule,
    RouterLink,
    MatMenuModule,
    MatBadgeModule,
  ],
  templateUrl: "./nav-bar.component.html",
  styleUrl: "./nav-bar.component.scss",
})
export class NavBarComponent {
  routerSub: Subscription | undefined;
  contextMenuPosition = { x: "0px", y: "0px" };
  menu = viewChild.required<MatMenuTrigger>(MatMenuTrigger);
  activeMenu = signal<MatMenu | null>(null);
  readonly clipboardHistoryService = inject(ClipboardHistoryService);
  readonly windowActionsService = inject(WindowActionsService);
  readonly router = inject(Router);
  readonly changeDetectorRef = inject(ChangeDetectorRef);
  readonly dialog = inject(MatDialog);
  readonly dropperService = inject(DropperService);

  onRightClick(event: MouseEvent, menu: MatMenu): void {
    this.contextMenuPosition.x = event.clientX + "px";
    this.contextMenuPosition.y = event.clientY + "px";
    event.preventDefault();
    this.activeMenu.set(menu);
    this.changeDetectorRef.detectChanges();
    this.menu().openMenu();
  }

  clearClipboard(): void {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Clear Clipboard`,
        message: `Are you sure you want to clear all?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.clipboardHistoryService.clear();
      }
    });
  }

  deleteAllFiles(): void {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Delete All Files`,
        message: `Are you sure you want to delete all files?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.dropperService.deleteAllFiles();
      }
    });
  }
}
