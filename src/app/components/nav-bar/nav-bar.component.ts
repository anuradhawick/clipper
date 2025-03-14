import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  signal,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { ClipboardHistoryService } from "../../services/clipboard-history.service";
import { WindowActionsService } from "../../services/window-actions.service";
import { NavigationEnd, Router, RouterLink } from "@angular/router";
import { Subscription } from "rxjs";

@Component({
  selector: "app-nav-bar",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [MatButtonModule, MatIconModule, RouterLink],
  templateUrl: "./nav-bar.component.html",
  styleUrl: "./nav-bar.component.scss",
})
export class NavBarComponent implements OnDestroy {
  routerSub: Subscription;
  showClear = signal(false);
  promptedClipboardDelete = signal(false);

  constructor(
    protected chs: ClipboardHistoryService,
    protected was: WindowActionsService,
    protected router: Router
  ) {
    this.routerSub = this.router.events.subscribe((event: any) => {
      if (event instanceof NavigationEnd) {
        this.showClear.set(event.url === "/");
      }
    });
  }

  ngOnDestroy(): void {
    this.routerSub.unsubscribe();
  }
}
