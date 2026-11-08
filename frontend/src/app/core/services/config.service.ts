import { Injectable, signal } from '@angular/core';
import { AppConfig } from '../models/config.model';

@Injectable({ providedIn: 'root' })
export class ConfigService {
  private readonly _config = signal<AppConfig | null>(null);
  readonly config = this._config.asReadonly();

  async load(): Promise<void> {
    const res = await fetch('/config.json', { cache: 'no-store' });
    if (!res.ok) throw new Error('Failed to load config');
    this._config.set(await res.json());
  }

  get value(): AppConfig {
    const c = this._config();
    if (!c) throw new Error('Config not loaded');
    return c;
  }
}
