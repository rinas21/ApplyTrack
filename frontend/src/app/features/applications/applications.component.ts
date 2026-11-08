import { Component, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { DatePipe } from '@angular/common';
import { ApiService } from '../../core/services/api.service';

@Component({
  standalone: true,
  imports: [FormsModule, DatePipe],
  template: `
  <div class="page">
    <div class="topbar"><div><h1>Applications</h1><div class="muted">Track status, notes, and pipeline history</div></div><button class="btn" (click)="showForm.set(!showForm())">{{showForm() ? 'Close' : 'Add application'}}</button></div>
    @if (showForm()) {
      <div class="card" style="margin-bottom:1rem;">
        <h3 style="margin-top:0;">{{editingId ? 'Edit' : 'New'}} application</h3>
        <div class="form-row">
          <div><label>Company *</label><input [(ngModel)]="form.company_name"></div>
          <div><label>Job title *</label><input [(ngModel)]="form.job_title"></div>
          <div><label>Status</label><select [(ngModel)]="form.status">@for(s of statuses;track s){<option [value]="s">{{s}}</option>}</select></div>
          <div><label>Source</label><input [(ngModel)]="form.application_source" placeholder="LinkedIn, Indeed, referral..."></div>
          <div><label>Location</label><input [(ngModel)]="form.location"></div>
          <div><label>Country</label><input [(ngModel)]="form.country"></div>
          <div><label>Date applied</label><input type="date" [(ngModel)]="form.date_applied"></div>
          <div><label>Follow-up date</label><input type="date" [(ngModel)]="form.follow_up_date"></div>
        </div>
        <div style="margin-top:.75rem;"><label>Job URL</label><input [(ngModel)]="form.job_url"></div>
        <div style="margin-top:.75rem;"><label>Notes</label><textarea rows="3" [(ngModel)]="form.notes"></textarea></div>
        <div style="margin-top:.75rem;display:flex;gap:.5rem;"><button class="btn" (click)="save()">Save</button><button class="btn secondary" (click)="reset()">Cancel</button></div>
      </div>
    }
    <div class="card">
      <div class="form-row" style="margin-bottom:1rem;">
        <input placeholder="Search company or title" [(ngModel)]="q" (keyup.enter)="load()">
        <select [(ngModel)]="statusFilter" (change)="load()"><option value="">All statuses</option>@for(s of statuses;track s){<option [value]="s">{{s}}</option>}</select>
        <button class="btn secondary" (click)="load()">Filter</button>
      </div>
      @if (loading()) { <p class="muted">Loading...</p> }
      @else if (!items().length) { <p class="muted">No applications found. Add one manually or save a job from the browser extension.</p> }
      @else {
        <table><thead><tr><th>Company</th><th>Role</th><th>Status</th><th>Applied</th><th>Source</th><th></th></tr></thead><tbody>
        @for (a of items(); track a.id) {
          <tr>
            <td>{{a.company_name}}<div class="muted" style="font-size:.85rem;">{{a.location || ''}}</div></td>
            <td>{{a.job_title}}</td>
            <td>
              <select [ngModel]="a.status" (ngModelChange)="changeStatus(a, $event)">
                @for(s of statuses;track s){<option [value]="s">{{s}}</option>}
              </select>
            </td>
            <td>{{a.date_applied || '—'}}</td><td>{{a.application_source || '—'}}</td>
            <td style="white-space:nowrap;"><button class="btn secondary" (click)="edit(a)">Edit</button> <button class="btn danger" (click)="remove(a)">Delete</button></td>
          </tr>
        }</tbody></table>
      }
    </div>
  </div>`
})
export class ApplicationsComponent implements OnInit {
  statuses = ['Saved','Applied','Assessment','Interview','Offer','Rejected','Withdrawn','No response'];
  items = signal<any[]>([]); loading = signal(true); showForm = signal(false);
  q=''; statusFilter=''; editingId: string | null = null;
  form: any = this.empty();
  constructor(private api: ApiService) {}
  empty() { return { company_name:'', job_title:'', status:'Saved', application_source:'', location:'', country:'', date_applied:'', follow_up_date:'', job_url:'', notes:'' }; }
  ngOnInit() { this.load(); }
  load() { this.loading.set(true); this.api.get<any>('/applications', { q: this.q, status: this.statusFilter, page_size: 100 }).subscribe({ next: d => { this.items.set(d.items||d.data?.items||[]); this.loading.set(false); }, error: () => this.loading.set(false) }); }
  save() { const obs = this.editingId ? this.api.put(`/applications/${this.editingId}`, this.form) : this.api.post('/applications', this.form); obs.subscribe({ next: () => { this.reset(); this.load(); } }); }
  edit(a:any){ this.form={...a}; this.editingId=a.id; this.showForm.set(true); }
  reset(){ this.form=this.empty(); this.editingId=null; this.showForm.set(false); }
  changeStatus(a:any, status:string){ this.api.patch(`/applications/${a.id}/status`, { status }).subscribe(() => this.load()); }
  remove(a:any){ if (confirm('Delete this application?')) this.api.delete(`/applications/${a.id}`).subscribe(() => this.load()); }
}
