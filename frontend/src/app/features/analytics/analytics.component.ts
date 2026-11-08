import { Component, OnInit, computed, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { ApiService } from '../../core/services/api.service';
import { ConfigService } from '../../core/services/config.service';

type BarItem = { label: string; count: number; pct: number; tone?: string };

@Component({
  standalone: true,
  template: `
  <div class="page">
    <div class="topbar">
      <div>
        <h1>Analytics</h1>
        <div class="muted">Metrics calculated from your PostgreSQL records. Saved items are tracked separately from submitted applications.</div>
      </div>
      <button class="btn secondary" type="button" (click)="exportCsv()" [disabled]="exporting()">{{ exporting() ? 'Exporting...' : 'Export CSV' }}</button>
    </div>

    @if (loading()) {
      <div class="card" aria-busy="true" aria-live="polite">Loading analytics...</div>
    } @else if (error()) {
      <div class="card alert-error" role="alert">{{ error() }}</div>
    } @else if (d()) {
      <section aria-label="Key metrics" class="kpi-grid">
        @for (c of cards(); track c.label) {
          <div class="card kpi-card">
            <div class="kpi-label">{{ c.label }}</div>
            <div class="kpi-value">{{ c.value }}</div>
            @if (c.sub) { <div class="kpi-sub">{{ c.sub }}</div> }
          </div>
        }
      </section>

      <div class="grid" style="grid-template-columns:repeat(auto-fit,minmax(320px,1fr));margin-top:1rem;">
        <div class="card">
          <h2 class="section-title">Pipeline distribution</h2>
          @if (!statusBars().length) {
            <p class="empty-state">No application data yet. Add or capture applications to see pipeline distribution.</p>
          } @else {
            <div class="bar-list" role="list" aria-label="Application counts by status">
              @for (b of statusBars(); track b.label) {
                <div class="bar-row" role="listitem">
                  <div class="bar-label"><span class="status-pill" [attr.data-status]="b.label">{{ b.label }}</span></div>
                  <div class="bar-track" aria-hidden="true"><div class="bar-fill" [class]="b.tone || ''" [style.width.%]="b.pct"></div></div>
                  <div class="bar-value">{{ b.count }}</div>
                </div>
              }
            </div>
          }
        </div>

        <div class="card">
          <h2 class="section-title">Submitted by month</h2>
          @if (!monthBars().length) {
            <p class="empty-state">No submitted applications in the recent monthly window.</p>
          } @else {
            <div class="bar-list" role="list" aria-label="Submitted applications by month">
              @for (b of monthBars(); track b.label) {
                <div class="bar-row" role="listitem">
                  <div class="bar-label" [title]="b.label">{{ b.label }}</div>
                  <div class="bar-track" aria-hidden="true"><div class="bar-fill" [style.width.%]="b.pct"></div></div>
                  <div class="bar-value">{{ b.count }}</div>
                </div>
              }
            </div>
          }
        </div>
      </div>

      <div class="grid" style="grid-template-columns:repeat(auto-fit,minmax(320px,1fr));margin-top:1rem;">
        <div class="card">
          <h2 class="section-title">Top companies</h2>
          @if (!companyBars().length) {
            <p class="empty-state">No company data available yet.</p>
          } @else {
            <div class="bar-list" role="list" aria-label="Applications by company">
              @for (b of companyBars(); track b.label) {
                <div class="bar-row" role="listitem">
                  <div class="bar-label" [title]="b.label">{{ b.label }}</div>
                  <div class="bar-track" aria-hidden="true"><div class="bar-fill" [style.width.%]="b.pct"></div></div>
                  <div class="bar-value">{{ b.count }}</div>
                </div>
              }
            </div>
          }
        </div>

        <div class="card">
          <h2 class="section-title">Top sources</h2>
          @if (!sourceBars().length) {
            <p class="empty-state">No source data available yet.</p>
          } @else {
            <div class="bar-list" role="list" aria-label="Applications by source">
              @for (b of sourceBars(); track b.label) {
                <div class="bar-row" role="listitem">
                  <div class="bar-label" [title]="b.label">{{ b.label }}</div>
                  <div class="bar-track" aria-hidden="true"><div class="bar-fill success" [style.width.%]="b.pct"></div></div>
                  <div class="bar-value">{{ b.count }}</div>
                </div>
              }
            </div>
          }
        </div>
      </div>

      <div class="card" style="margin-top:1rem;">
        <h2 class="section-title">Metric definitions</h2>
        @if (definitionEntries().length) {
          <dl class="def-grid">
            @for (e of definitionEntries(); track e.key) {
              <div class="def-item"><dt>{{ e.key }}</dt><dd>{{ e.value }}</dd></div>
            }
          </dl>
        } @else {
          <p class="empty-state">No metric definitions returned by the API.</p>
        }
      </div>
    }
  </div>`
})
export class AnalyticsComponent implements OnInit {
  loading = signal(true);
  exporting = signal(false);
  error = signal('');
  d = signal<any>(null);

  cards = computed(() => {
    const d = this.d(); if (!d) return [];
    const t = d.totals || {}; const r = d.rates || {};
    return [
      { label: 'Total tracked', value: t.total ?? 0, sub: `${t.saved ?? 0} saved` },
      { label: 'Submitted', value: t.submitted ?? 0, sub: `${t.pending ?? 0} active/pending` },
      { label: 'Interviews', value: t.interviews ?? 0, sub: `${r.interview_conversion_rate ?? 0}% conversion` },
      { label: 'Offers', value: t.offers ?? 0, sub: `${r.offer_rate ?? 0}% offer rate` },
      { label: 'Rejected', value: t.rejections ?? 0, sub: `${r.rejection_rate ?? 0}% rejection rate` },
      { label: 'Response rate', value: `${r.response_rate ?? 0}%`, sub: 'Approximate (see definitions)' },
    ];
  });

  statusBars = computed(() => this.toBars((this.d()?.status_distribution || []).map((x: any) => ({ label: x.status, count: x.count })), true));
  monthBars = computed(() => this.toBars((this.d()?.applications_by_month || []).slice().reverse().map((x: any) => ({ label: x.month, count: x.count }))));
  companyBars = computed(() => this.toBars((this.d()?.top_companies || []).map((x: any) => ({ label: x.company, count: x.count }))));
  sourceBars = computed(() => this.toBars((this.d()?.sources || []).map((x: any) => ({ label: x.source || 'Unknown', count: x.count }))));
  definitionEntries = computed(() => Object.entries(this.d()?.definitions || {}).map(([key, value]) => ({ key: key.replace(/_/g, ' '), value: String(value) })));

  constructor(private api: ApiService, private cfg: ConfigService, private http: HttpClient) {}

  ngOnInit() {
    this.api.get<any>('/analytics/summary').subscribe({
      next: (d: any) => { this.d.set(d); this.loading.set(false); },
      error: (e: any) => { this.error.set(e?.error?.message || e?.message || 'Failed to load analytics.'); this.loading.set(false); }
    });
  }

  private toBars(items: Array<{ label: string; count: number }>, statusTones = false): BarItem[] {
    const max = Math.max(1, ...items.map(i => Number(i.count) || 0));
    return items.map(i => ({
      label: i.label || 'Unknown',
      count: Number(i.count) || 0,
      pct: Math.max(3, Math.round(((Number(i.count) || 0) / max) * 100)),
      tone: statusTones ? this.toneFor(i.label) : undefined,
    }));
  }

  private toneFor(status: string): string | undefined {
    if (status === 'Offer') return 'success';
    if (status === 'Interview' || status === 'Assessment') return 'warn';
    if (status === 'Rejected') return 'danger';
    return undefined;
  }

  exportCsv() {
    this.exporting.set(true);
    this.error.set('');
    this.http.get(`${this.cfg.value.apiBaseUrl}/analytics/export.csv`, { responseType: 'blob' }).subscribe({
      next: (blob) => {
        const u = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = u; a.download = 'applytrack-export.csv'; a.click();
        URL.revokeObjectURL(u);
        this.exporting.set(false);
      },
      error: (e: any) => {
        this.exporting.set(false);
        this.error.set(e?.error?.message || 'CSV export failed. Please ensure you are signed in and try again.');
      }
    });
  }
}
