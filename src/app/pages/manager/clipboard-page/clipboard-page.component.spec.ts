import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ClipboardPageComponent } from './clipboard-page.component';

describe('ClipboardPageComponent', () => {
  let component: ClipboardPageComponent;
  let fixture: ComponentFixture<ClipboardPageComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ClipboardPageComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ClipboardPageComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
