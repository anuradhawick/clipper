import { ComponentFixture, TestBed } from "@angular/core/testing";

import { ClipboardItemsPageComponent } from "./clipboard-items-page.component";

describe("ClipboardItemsPageComponent", () => {
  let component: ClipboardItemsPageComponent;
  let fixture: ComponentFixture<ClipboardItemsPageComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ClipboardItemsPageComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(ClipboardItemsPageComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
