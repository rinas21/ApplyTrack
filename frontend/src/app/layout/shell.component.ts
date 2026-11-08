import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { AuthService } from '../core/auth/auth.service';

@Component({
  selector: 'app-shell',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  template: `
  <div class="layout">
    <aside class="sidebar card">
      <div class="sidebar-brand">
        <div class="title">ApplyTrack</div>
        <div class="muted" style="font-size:.9rem;margin-top:.2rem;">Job search, organized</div>
      </div>
      <nav aria-label="Primary" style="display:flex;flex-direction:column;gap:.3rem;">
        @for (item of nav; track item.path) {
          <a class="nav-link" [routerLink]="item.path" routerLinkActive="active">{{item.label}}</a>
        }
      </nav>
      <div class="sidebar-footer">
        <div class="muted" style="font-size:.85rem;margin-bottom:.65rem;word-break:break-word;">{{auth.profile()?.email || auth.profile()?.name || 'Signed in'}}</div>
        <button class="btn secondary btn-block" type="button" (click)="auth.logout()">Log out</button>
      </div>
    </aside>
    <main class="main-content"><router-outlet /></main>
  </div>`
})
export class ShellComponent {
  nav = [
    { path: '/dashboard', label: 'Overview' },
    { path: '/applications', label: 'Applications' },
    { path: '/companies', label: 'Companies' },
    { path: '/jobs', label: 'Jobs' },
    { path: '/analytics', label: 'Analytics' },
    { path: '/email-activity', label: 'Email Activity' },
    { path: '/reminders', label: 'Reminders' },
    { path: '/settings', label: 'Settings' },
  ];
  constructor(public auth: AuthService) {}
}
