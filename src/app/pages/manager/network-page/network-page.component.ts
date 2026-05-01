import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  signal,
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatIconModule } from "@angular/material/icon";
import { MatInputModule } from "@angular/material/input";
import { MatTooltipModule } from "@angular/material/tooltip";
import { NetworkPeer, NetworkService } from "../../../services/network.service";

@Component({
  selector: "app-network-page",
  imports: [
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatTooltipModule,
  ],
  templateUrl: "./network-page.component.html",
  styleUrl: "./network-page.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkPageComponent {
  protected readonly networkService = inject(NetworkService);
  protected readonly authorizedPeers = computed(() =>
    this.networkService.peers().filter((peer) => peer.authorized),
  );
  protected readonly pendingPeers = computed(() =>
    this.networkService.peers().filter((peer) => !peer.authorized),
  );
  protected readonly peerOtps = signal<Partial<Record<string, string>>>({});
  protected readonly busyPeerId = signal<string | null>(null);

  protected trackByPeerId(_: number, peer: NetworkPeer): string {
    return peer.id;
  }

  protected setPeerOtp(peerId: string, value: string): void {
    this.peerOtps.update((otps) => ({
      ...otps,
      [peerId]: value.replace(/\D/g, "").slice(0, 6),
    }));
  }

  protected async authorize(peer: NetworkPeer): Promise<void> {
    const otp = this.peerOtps()[peer.id] ?? "";
    if (otp.length !== 6) {
      return;
    }

    this.busyPeerId.set(peer.id);
    try {
      await this.networkService.authorizePeer(peer.id, otp);
      this.peerOtps.update((otps) => {
        const next = { ...otps };
        delete next[peer.id];
        return next;
      });
    } finally {
      this.busyPeerId.set(null);
    }
  }

  protected async revoke(peer: NetworkPeer): Promise<void> {
    this.busyPeerId.set(peer.id);
    try {
      await this.networkService.revokePeer(peer.id);
    } finally {
      this.busyPeerId.set(null);
    }
  }
}
