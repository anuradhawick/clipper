import { Injectable, OnDestroy, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface NetworkStatus {
  running: boolean;
  local_name: string;
  otp: string | null;
}

export interface NetworkPeer {
  id: string;
  name: string;
  authorized: boolean;
}

@Injectable({
  providedIn: "root",
})
export class NetworkService implements OnDestroy {
  readonly running = signal(false);
  readonly localName = signal("Clipper");
  readonly otp = signal<string | null>(null);
  readonly peers = signal<NetworkPeer[]>([]);
  readonly loading = signal(false);

  private unlistenNetworkStatus: UnlistenFn | undefined;
  private unlistenNetworkPeers: UnlistenFn | undefined;

  constructor() {
    listen<boolean>("net_status_changed", (event) => {
      this.running.set(event.payload);
      void this.refreshStatus();
      void this.refreshPeers();
    }).then((unlisten) => (this.unlistenNetworkStatus = unlisten));

    listen("net_peers_updated", () => {
      void this.refreshPeers();
    }).then((unlisten) => (this.unlistenNetworkPeers = unlisten));

    void this.refresh();
  }

  ngOnDestroy(): void {
    if (this.unlistenNetworkStatus) {
      this.unlistenNetworkStatus();
    }
    if (this.unlistenNetworkPeers) {
      this.unlistenNetworkPeers();
    }
  }

  async refresh(): Promise<void> {
    this.loading.set(true);
    try {
      await Promise.all([this.refreshStatus(), this.refreshPeers()]);
    } finally {
      this.loading.set(false);
    }
  }

  async refreshStatus(): Promise<void> {
    const status = await invoke<NetworkStatus>("net_get_status", {});
    this.running.set(status.running);
    this.localName.set(status.local_name);
    this.otp.set(status.otp);
  }

  async refreshPeers(): Promise<void> {
    const peers = await invoke<NetworkPeer[]>("net_list_peers", {});
    this.peers.set(peers);
  }

  async generateOtp(): Promise<void> {
    const otp = await invoke<string>("net_generate_otp", {});
    this.otp.set(otp);
  }

  async authorizePeer(peerId: string, otp: string): Promise<void> {
    await invoke<void>("net_authorize_peer", {
      peerId,
      otp,
    });
    await this.refreshPeers();
  }

  async revokePeer(peerId: string): Promise<void> {
    await invoke<void>("net_revoke_peer", {
      peerId,
    });
    await this.refreshPeers();
  }

  async toggle(): Promise<void> {
    if (this.running()) {
      this.running.set(false);
      this.peers.set([]);
      try {
        await invoke<void>("net_stop", {});
      } catch (error) {
        await this.refresh();
        throw error;
      }
    } else {
      this.running.set(true);
      try {
        await invoke<void>("net_start", {});
        await this.refresh();
      } catch (error) {
        await this.refresh();
        throw error;
      }
    }
  }
}
