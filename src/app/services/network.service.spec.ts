import { TestBed } from "@angular/core/testing";

import { NetworkService } from "./network.service";

interface TauriInternalsMock {
  invoke: jasmine.Spy;
  transformCallback: jasmine.Spy;
}

describe("NetworkService", () => {
  let service: NetworkService;
  let tauriInternals: TauriInternalsMock;

  beforeEach(() => {
    tauriInternals = {
      invoke: jasmine.createSpy("invoke").and.callFake((command: string) => {
        switch (command) {
          case "net_get_status":
            return Promise.resolve({
              running: false,
              local_name: "Clipper",
              otp: null,
            });
          case "net_list_peers":
            return Promise.resolve([]);
          default:
            return Promise.resolve(undefined);
        }
      }),
      transformCallback: jasmine
        .createSpy("transformCallback")
        .and.returnValue(1),
    };
    (
      window as Window & { __TAURI_INTERNALS__: TauriInternalsMock }
    ).__TAURI_INTERNALS__ = tauriInternals;

    TestBed.configureTestingModule({});
    service = TestBed.inject(NetworkService);
  });

  it("should be created", () => {
    expect(service).toBeTruthy();
  });
});
