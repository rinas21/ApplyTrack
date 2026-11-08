import { Component, OnInit, computed, signal } from '@angular/core';
import { DatePipe } from '@angular/common';
import { RouterLink } from '@angular/router';
import { ApiService } from '../../core/services/api.service';

type BarItem = { label: string; count: number; pct: number; tone?: string };

@Component({
  standalone: true,
  imports: [DatePipe, RouterLink],
  template: `
  <div class="page">
    <div class="topbar">
      <div>
        <h1>Overview</h1>
        <div class="muted">A clear snapshot of your job search, calculated from your real account data.</div>
      </div>
      <a class="btn" routerLink="/applications">Manage applications</a>
    </div>

    @if (loading()) {
      <div class="card" aria-busy="true">Loading dashboard...</div>
    } @else if (error()) {
      <div class="card alert-error" role="alert">{{ error() }}</div>
    } @else if (data()) {
      <section aria-label="Key metrics" class="kpi-grid" style="margin-bottom:1rem;">
        @for (card of cards(); track card.label) {
          <div class="card kpi-card">
            <div class="kpi-label">{{ card.label }}</div>
            <div class="kpi-value">{{ card.value }}</div>
            @if (card.sub) { <div class="kpi-sub">{{ card.sub }}</div> }
          </div>
        }
      </section>

      <div class="grid" style="grid-template-columns:repeat(auto-fit,minmax(320px,1fr));">
        <div class="card">
          <h2 class="section-title">Recent activity</h2>
          @if (!data().recent_activity?.length) {
            <p class="empty-state">No applications yet. Save a job, capture one from the extension, or add an application to get started.</p>
          } @else {
            <div style="overflow:auto;">
              <table>
                <thead><tr><th>Company</th><th>Role</th><th>Status</th><th>Updated</th></tr></thead>
                <tbody>
                @for (r of data().recent_activity; track r.id) {
                  <tr>
                    <td>{{ r.company_name }}</td>
                    <td>{{ r.job_title }}</td>
                    <td><span class="status-pill" [attr.data-status]="r.status">{{ r.status }}</span></td>
                    <td class="muted">{{ r.updated_at | date:'mediumDate' }}</td>
                  </tr>
                }
                </tbody>
              </table>
            </div>
          }
        </div>

        <div class="card">
          <h2 class="section-title">Pipeline</h2>
          @if (!statusBars().length) {
            <p class="empty-state">Pipeline data will appear here once you track applications.</p>
          } @else {
            <div class="bar-list" role="list" aria-label="Pipeline by status">
              @for (b of statusBars(); track b.label) {
                <div class="bar-row" role="listitem">
                  <div class="bar-label"><span class="status-pill" [attr.data-status]="b.label">{{ b.label }}</span></div>
                  <div class="bar-track" aria-hidden="true"><div class="bar-fill" [class]="b.tone || ''" [style.width.%]="b.pct"></div></div>
                  <div class="bar-value">{{ b.count }}</div>
                </div>
              }
            </div>
          }

          <h2 class="section-title" style="margin-top:1.25rem;">Top sources</h2>
          @if (!sourceBars().length) {
            <p class="empty-state">No source data yet.</p>
          } @else {
            <div class="bar-list" role="list" aria-label="Top sources">
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
    }
  </div>`
})
export class DashboardComponent implements OnInit {
  loading = signal(true);
  error = signal('');
  data = signal<any>(null);

  cards = computed(() => {
    const d = this.data(); if (!d) return [];
    const t = d.totals || {}; const r = d.rates || {};
    return [
      { label: 'Saved', value: t.saved ?? 0, sub: 'Tracked opportunities' },
      { label: 'Submitted', value: t.submitted ?? 0, sub: 'Excludes saved-only' },
      { label: 'Interviews', value: t.interviews ?? 0, sub: `${r.interview_conversion_rate ?? 0}% conversion` },
      { label: 'Offers', value: t.offers ?? 0, sub: `${r.offer_rate ?? 0}% offer rate` },
      { label: 'Rejected', value: t.rejections ?? 0, sub: `${r.rejection_rate ?? 0}% of submitted` },
      { label: 'Response rate', value: `${r.response_rate ?? 0}%`, sub: 'Approximate' },
    ];
  });

  statusBars = computed(() => this.toBars((this.data()?.status_distribution || []).map((x: any) => ({ label: x.status, count: x.count })), true));
  sourceBars = computed(() => this.toBars((this.data()?.sources || []).slice(0, 6).map((x: any) => ({ label: x.source || 'Unknown', count: x.count }))));

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.api.get<any>('/analytics/summary').subscribe({
      next: d => { this.data.set(d); this.loading.set(false); },
      error: e => { this.error.set(e?.error?.message || e?.message || 'Failed to load dashboard. Please sign in and ensure the API is available.'); this.loading.set(false); }
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
}
