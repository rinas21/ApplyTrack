import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { AuthService } from '../../core/auth/auth.service';

@Component({
  standalone: true,
  imports: [RouterLink],
  template: `
  <div style="min-height:100vh;display:grid;place-items:center;padding:1.5rem;">
    <div class="card" style="max-width:520px;width:100%;text-align:center;">
      <div class="muted" style="margin-bottom:.5rem;">ApplyTrack</div>
      <p style="margin:0 0 1rem;font-size:1.05rem;">{{message}}</p>
      @if (failed) { <a class="btn secondary" routerLink="/login">Back to sign in</a> }
    </div>
  </div>`
})
export class CallbackComponent implements OnInit {
  message = 'Completing sign-in...';
  failed = false;
  constructor(private route: ActivatedRoute, private auth: AuthService, private router: Router) {}
  async ngOnInit() {
    const code = this.route.snapshot.queryParamMap.get('code');
    const state = this.route.snapshot.queryParamMap.get('state');
    const error = this.route.snapshot.queryParamMap.get('error');
    if (error) { this.message = `Authentication failed: ${error}`; this.failed = true; return; }
    if (!code || !state) { this.message = 'Missing callback parameters. Please try signing in again.'; this.failed = true; return; }
    try { await this.auth.handleCallback(code, state); }
    catch (e: any) { this.message = e?.message || 'Authentication failed.'; this.failed = true; }
  }
}
