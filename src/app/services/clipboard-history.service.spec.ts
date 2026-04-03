import { TestBed } from "@angular/core/testing";

import { ClipboardHistoryService } from "./clipboard-history.service";

describe("ClipboardHistoryService", () => {
  let service: ClipboardHistoryService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ClipboardHistoryService);
  });

  it("should be created", () => {
    expect(service).toBeTruthy();
  });
});
