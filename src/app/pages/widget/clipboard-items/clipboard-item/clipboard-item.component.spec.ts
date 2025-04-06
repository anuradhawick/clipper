import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ClipboardItemComponent } from './clipboard-item.component';

describe('ClipboardItemComponent', () => {
  let component: ClipboardItemComponent;
  let fixture: ComponentFixture<ClipboardItemComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ClipboardItemComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ClipboardItemComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
