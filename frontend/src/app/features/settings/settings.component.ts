import { Component, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../core/services/api.service';
import { AuthService } from '../../core/auth/auth.service';

@Component({
  standalone: true,
  imports: [FormsModule],
  template: `
  <div class="page">
    <div class="topbar"><div><h1>Settings</h1><div class="muted">Profile, preferences, email connections, and data controls</div></div></div>
    <div class="grid" style="grid-template-columns:1fr 1fr;">
      <div class="card">
        <h3 style="margin-top:0;">Profile</h3>
        <p><strong>User:</strong> {{auth.profile()?.name || 'Authenticated user'}}</p>
        <p><strong>Email:</strong> {{auth.profile()?.email || '—'}}</p>
        <p class="muted">Identity is managed by Keycloak. Application data is isolated per authenticated user.</p>
      </div>
      <div class="card">
        <h3 style="margin-top:0;">Preferences</h3>
        <label>Timezone</label><input [(ngModel)]="prefs.timezone" placeholder="UTC">
        <label style="margin-top:.5rem;">Follow-up days</label><input type="number" [(ngModel)]="prefs.follow_up_days">
        <label style="margin-top:.5rem;display:flex;gap:.5rem;align-items:center;color:var(--text);"><input type="checkbox" style="width:auto" [(ngModel)]="prefs.email_notifications"> Email notifications (when configured)</label>
        <button class="btn" style="margin-top:.75rem;" (click)="savePrefs()">Save preferences</button>
      </div>
      <div class="card" style="grid-column:1/-1;">
        <h3 style="margin-top:0;">Connected email accounts</h3>
        <p class="muted">Gmail integration uses Google OAuth on the backend. Refresh tokens are encrypted server-side and never exposed to the browser. Real sync requires Google OAuth credentials.</p>
        <div style="display:flex;gap:.5rem;margin-bottom:1rem;"><button class="btn" (click)="connectGmail()">Connect Gmail</button><button class="btn secondary" (click)="loadAccounts()">Refresh</button></div>
        @for (a of accounts(); track a.id) {
          <div style="display:flex;justify-content:space-between;align-items:center;padding:.6rem 0;border-bottom:1px solid var(--border);">
            <div><strong>{{a.provider}}</strong> {{a.email_address || ''}} <span class="pill">{{a.status}}</span></div>
            <div><button class="btn secondary" (click)="sync(a)">Sync</button> <button class="btn danger" (click)="disconnect(a)">Disconnect & delete data</button></div>
          </div>
        }
      </div>
      <div class="card"><h3 style="margin-top:0;">Subscription</h3><p><strong>Plan:</strong> {{sub()?.plan || 'free'}} ({{sub()?.status || 'active'}})</p><p class="muted">Billing provider integration is isolated for future setup. Usage limits are modeled in the backend.</p></div>
      <div class="card"><h3 style="margin-top:0;">Data controls</h3><button class="btn secondary" (click)="exportData()">Export my data (JSON)</button><p class="muted" style="margin-top:.75rem;">Account deletion requires backend confirmation flow and should remove user-owned records per policy; implement full deletion endpoint before production.</p></div>
    </div>
  </div>`
})
export class SettingsComponent implements OnInit {
  prefs: any = { timezone: 'UTC', follow_up_days: 7, email_notifications: false };
  accounts = signal<any[]>([]); sub = signal<any>(null);
  constructor(private api: ApiService, public auth: AuthService) {}
  ngOnInit(){ this.api.get('/settings/preferences').subscribe(p=>this.prefs=p||this.prefs); this.loadAccounts(); this.api.get('/settings/subscription').subscribe(s=>this.sub.set(s)); }
  loadAccounts(){ this.api.get<any[]>('/email/accounts').subscribe(a=>this.accounts.set(a||[])); }
  savePrefs(){ this.api.put('/settings/preferences', this.prefs).subscribe(); }
  connectGmail(){ this.api.get<any>('/email/gmail/auth-url').subscribe({ next: (r: any) => window.location.href=r.url, error: (e: any) => alert(e?.error?.message || 'Gmail not configured. Set GOOGLE_* env vars.') }); }
  sync(a:any){ this.api.post<any>(`/email/accounts/${a.id}/sync`, {}).subscribe({ next: (r: any)=>alert(`Synced ${r.synced_messages ?? 0} messages`), error: (e: any)=>alert(e?.error?.message||'Sync failed') }); }
  disconnect(a:any){ if(confirm('Disconnect account and delete synchronized email data?')) this.api.delete(`/email/accounts/${a.id}`).subscribe(()=>this.loadAccounts()); }
  exportData(){ this.api.get<any>('/settings/export').subscribe((d: any)=>{ const blob=new Blob([JSON.stringify(d,null,2)],{type:'application/json'}); const u=URL.createObjectURL(blob); const a=document.createElement('a'); a.href=u; a.download='applytrack-export.json'; a.click(); URL.revokeObjectURL(u); }); }
}
