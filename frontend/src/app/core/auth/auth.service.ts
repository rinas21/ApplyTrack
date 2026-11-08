import { Injectable, computed, signal } from '@angular/core';
import { Router } from '@angular/router';
import { ConfigService } from '../services/config.service';

interface TokenResponse { access_token: string; refresh_token?: string; id_token?: string; expires_in: number; }

function base64UrlEncode(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let str = '';
  for (const b of bytes) str += String.fromCharCode(b);
  return btoa(str).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
}
async function sha256(text: string): Promise<ArrayBuffer> {
  return crypto.subtle.digest('SHA-256', new TextEncoder().encode(text));
}
function randomString(len = 64): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  const bytes = crypto.getRandomValues(new Uint8Array(len));
  return Array.from(bytes).map(b => chars[b % chars.length]).join('');
}

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly _token = signal<string | null>(sessionStorage.getItem('at.access'));
  private readonly _profile = signal<{ sub?: string; email?: string; name?: string } | null>(null);
  readonly isAuthenticated = computed(() => !!this._token());
  readonly token = this._token.asReadonly();
  readonly profile = this._profile.asReadonly();

  constructor(private cfg: ConfigService, private router: Router) {
    const t = this._token();
    if (t) this.parseProfile(t);
  }

  private issuer(): string {
    const c = this.cfg.value;
    return `${c.keycloakUrl}/realms/${c.keycloakRealm}`;
  }

  async login(returnUrl = '/'): Promise<void> {
    const c = this.cfg.value;
    const verifier = randomString(64);
    const challenge = base64UrlEncode(await sha256(verifier));
    const state = randomString(24);
    sessionStorage.setItem('pkce.verifier', verifier);
    sessionStorage.setItem('pkce.state', state);
    sessionStorage.setItem('auth.return', returnUrl);
    const redirectUri = `${window.location.origin}/callback`;
    const url = new URL(`${this.issuer()}/protocol/openid-connect/auth`);
    url.searchParams.set('client_id', c.keycloakClientId);
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('scope', 'openid profile email');
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('code_challenge', challenge);
    url.searchParams.set('code_challenge_method', 'S256');
    url.searchParams.set('state', state);
    window.location.href = url.toString();
  }

  async handleCallback(code: string, state: string): Promise<void> {
    if (state !== sessionStorage.getItem('pkce.state')) throw new Error('Invalid auth state');
    const verifier = sessionStorage.getItem('pkce.verifier');
    if (!verifier) throw new Error('Missing PKCE verifier');
    const c = this.cfg.value;
    const redirectUri = `${window.location.origin}/callback`;
    const body = new URLSearchParams({
      grant_type: 'authorization_code',
      client_id: c.keycloakClientId,
      code,
      redirect_uri: redirectUri,
      code_verifier: verifier,
    });
    const res = await fetch(`${this.issuer()}/protocol/openid-connect/token`, {
      method: 'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded' }, body
    });
    if (!res.ok) throw new Error(`Token exchange failed: ${await res.text()}`);
    const tokens = await res.json() as TokenResponse;
    sessionStorage.setItem('at.access', tokens.access_token);
    if (tokens.refresh_token) sessionStorage.setItem('at.refresh', tokens.refresh_token);
    if (tokens.id_token) sessionStorage.setItem('at.id', tokens.id_token);
    this._token.set(tokens.access_token);
    this.parseProfile(tokens.access_token);
    sessionStorage.removeItem('pkce.verifier'); sessionStorage.removeItem('pkce.state');
    const ret = sessionStorage.getItem('auth.return') || '/';
    sessionStorage.removeItem('auth.return');
    await this.router.navigateByUrl(ret);
  }

  private parseProfile(token: string): void {
    try {
      const payload = JSON.parse(atob(token.split('.')[1].replace(/-/g, '+').replace(/_/g, '/')));
      this._profile.set({ sub: payload.sub, email: payload.email, name: payload.name || payload.preferred_username });
    } catch { /* ignore */ }
  }

  clearSession(): void {
    sessionStorage.removeItem('at.access');
    sessionStorage.removeItem('at.refresh');
    sessionStorage.removeItem('at.id');
    this._token.set(null);
    this._profile.set(null);
  }

  logout(): void {
    const c = this.cfg.value;
    const idHint = sessionStorage.getItem('at.id');
    this.clearSession();
    const url = new URL(`${this.issuer()}/protocol/openid-connect/logout`);
    url.searchParams.set('client_id', c.keycloakClientId);
    url.searchParams.set('post_logout_redirect_uri', window.location.origin + '/login');
    if (idHint) url.searchParams.set('id_token_hint', idHint);
    window.location.href = url.toString();
  }

  getAccessToken(): string | null { return this._token(); }
}
