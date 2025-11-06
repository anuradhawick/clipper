import { TestBed } from "@angular/core/testing";

import { DropperService } from "./dropper.service";

describe("DropperService", () => {
  let service: DropperService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(DropperService);
  });

  it("should be created", () => {
    expect(service).toBeTruthy();
  });
});
