import { ComponentFixture, TestBed } from "@angular/core/testing";

import { DragDropOverlayComponent } from "./drag-drop-overlay.component";

describe("DragDropOverlayComponent", () => {
  let component: DragDropOverlayComponent;
  let fixture: ComponentFixture<DragDropOverlayComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DragDropOverlayComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(DragDropOverlayComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
