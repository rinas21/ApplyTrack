import { Component, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { AuthService } from '../../core/auth/auth.service';
import { ConfigService } from '../../core/services/config.service';

@Component({
  selector: 'app-login',
  standalone: true,
  template: `
  <div class="auth-page">
    <div class="auth-shell">
      <div class="auth-brand">
        <div class="brand-mark">AT</div>
        <h1>ApplyTrack</h1>
        <p class="muted">A focused workspace for applications, job capture, and recruitment follow-through.</p>
        <ul class="auth-points muted">
          <li>Track every application and status change</li>
          <li>Capture roles from your browser extension</li>
          <li>Keep your data isolated to your account</li>
        </ul>
      </div>
      <div class="card auth-card">
        <h2 style="margin:0 0 .25rem;font-size:1.25rem;">Sign in</h2>
        <p class="muted" style="margin-top:0;">Use your organization's Keycloak account to continue.</p>
        @if (!ready) { <p class="muted">Loading configuration...</p> }
        @else {
          <button class="btn btn-block" (click)="login()">Continue with Keycloak</button>
          <p class="muted auth-note">Secure sign-in via Authorization Code + PKCE. Your data is scoped to your identity.</p>
        }
      </div>
    </div>
  </div>`
})
export class LoginComponent implements OnInit {
  ready = false;
  constructor(private auth: AuthService, private route: ActivatedRoute, private cfg: ConfigService) {}
  ngOnInit() { this.ready = !!this.cfg.config(); }
  login() { this.auth.login(this.route.snapshot.queryParamMap.get('returnUrl') || '/'); }
}
