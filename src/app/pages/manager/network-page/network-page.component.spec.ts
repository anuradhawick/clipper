import { ComponentFixture, TestBed } from "@angular/core/testing";
import { signal } from "@angular/core";

import { NetworkPageComponent } from "./network-page.component";
import { NetworkService } from "../../../services/network.service";

describe("NetworkPageComponent", () => {
  let component: NetworkPageComponent;
  let fixture: ComponentFixture<NetworkPageComponent>;
  const networkServiceStub = {
    running: signal(false),
    localName: signal("Clipper"),
    otp: signal<string | null>(null),
    peers: signal([]),
    refresh: () => Promise.resolve(),
    generateOtp: () => Promise.resolve(),
    toggle: () => Promise.resolve(),
    authorizePeer: () => Promise.resolve(),
    revokePeer: () => Promise.resolve(),
  };

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [NetworkPageComponent],
      providers: [{ provide: NetworkService, useValue: networkServiceStub }],
    }).compileComponents();

    fixture = TestBed.createComponent(NetworkPageComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
