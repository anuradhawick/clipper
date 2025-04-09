import { ComponentFixture, TestBed } from '@angular/core/testing';

import { NavPaneComponent } from './nav-pane.component';

describe('NavPaneComponent', () => {
  let component: NavPaneComponent;
  let fixture: ComponentFixture<NavPaneComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [NavPaneComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(NavPaneComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
