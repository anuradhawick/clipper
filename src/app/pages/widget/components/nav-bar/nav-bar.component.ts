import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ElementRef,
  effect,
  inject,
  OnDestroy,
  signal,
  viewChild,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { ClipboardHistoryService } from "../../../../services/clipboard-history.service";
import { WindowActionsService } from "../../../../services/window-actions.service";
import {
  ActivatedRoute,
  Event,
  EventType,
  Router,
  RouterLink,
} from "@angular/router";
import { Subscription } from "rxjs";
import { MatMenu, MatMenuModule, MatMenuTrigger } from "@angular/material/menu";
import { MatDialog } from "@angular/material/dialog";
import { ActionConfirmationDialogComponent } from "../../../../components/action-confirmation-dialog/action-confirmation-dialog.component";
import { MatBadgeModule } from "@angular/material/badge";
import { DropperService } from "../../../../services/dropper.service";
import { Location, TitleCasePipe } from "@angular/common";
import { MatTooltipModule } from "@angular/material/tooltip";
import { MatFormField } from "@angular/material/select";
import { MatInputModule } from "@angular/material/input";
import { NotesService } from "../../../../services/notes.service";

@Component({
  selector: "app-nav-bar",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    MatButtonModule,
    MatIconModule,
    RouterLink,
    MatMenuModule,
    MatBadgeModule,
    TitleCasePipe,
    MatTooltipModule,
    MatFormField,
    MatInputModule,
  ],
  templateUrl: "./nav-bar.component.html",
  styleUrl: "./nav-bar.component.scss",
})
export class NavBarComponent implements OnDestroy {
  private searchInputRef =
    viewChild<ElementRef<HTMLInputElement>>("searchInput");
  routerSubscription: Subscription;
  contextMenuPosition = { x: "0px", y: "0px" };
  menu = viewChild.required<MatMenuTrigger>(MatMenuTrigger);
  activeMenu = signal<MatMenu | null>(null);
  pageTitle = signal<string>("");
  readonly clipboardHistoryService = inject(ClipboardHistoryService);
  readonly notesService = inject(NotesService);
  readonly windowActionsService = inject(WindowActionsService);
  readonly changeDetectorRef = inject(ChangeDetectorRef);
  readonly dialog = inject(MatDialog);
  readonly dropperService = inject(DropperService);
  readonly router = inject(Router);
  readonly activatedRoute = inject(ActivatedRoute);
  readonly location = inject(Location);
  protected showSearch = signal(false);
  protected searchable = signal(false);
  protected searchQuery = signal("");

  constructor() {
    this.searchQuery.set(
      this.activatedRoute.snapshot.queryParamMap.get("search") ?? "",
    );

    effect(() => {
      const nextSearch = this.searchQuery();
      const currentSearch =
        this.activatedRoute.snapshot.queryParamMap.get("search") ?? "";

      if (nextSearch === currentSearch) {
        return;
      }

      void this.router.navigate([], {
        relativeTo: this.activatedRoute,
        queryParams: { search: nextSearch || null },
        queryParamsHandling: "merge",
        replaceUrl: true,
      });
    });

    this.routerSubscription = this.router.events.subscribe((event: Event) => {
      console.log("Router event:", event);
      switch (event.type) {
        case EventType.NavigationEnd:
          const url = this.location.path();
          const title = url.split("/")[2].split("?")[0];
          this.pageTitle.set(title);
          this.searchable.set(["notes", "clipboard"].includes(title));
          break;
      }
    });
  }

  protected toggleSearch(): void {
    const shouldShow = !this.showSearch();
    this.showSearch.set(shouldShow);

    if (shouldShow) {
      setTimeout(() => this.searchInputRef()?.nativeElement.focus());
    } else {
      this.searchQuery.set("");
    }
  }

  protected clearSearch(searchInput: HTMLInputElement) {
    searchInput.value = "";
    this.searchQuery.set("");
  }

  onRightClick(event: MouseEvent, menu: MatMenu): void {
    this.contextMenuPosition.x = event.clientX + "px";
    this.contextMenuPosition.y = event.clientY + "px";
    this.activeMenu.set(menu);
    this.changeDetectorRef.markForCheck();
    setTimeout(() => {
      this.menu().openMenu();
    });
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

  deleteAllNotes(): void {
    const dialogRef = this.dialog.open(ActionConfirmationDialogComponent, {
      data: {
        title: `Delete All Notes`,
        message: `Are you sure you want to delete all notes?`,
      },
    });
    dialogRef.afterClosed().subscribe((result) => {
      if (result) {
        this.notesService.deleteAll();
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

  ngOnDestroy(): void {
    this.routerSubscription.unsubscribe();
  }
}
