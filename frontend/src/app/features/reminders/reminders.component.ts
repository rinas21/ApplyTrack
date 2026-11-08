import { Component, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { DatePipe } from '@angular/common';
import { ApiService } from '../../core/services/api.service';

@Component({ standalone:true, imports:[FormsModule, DatePipe], template:`<div class="page"><div class="topbar"><div><h1>Reminders</h1><div class="muted">Follow-ups and suggestions for applications without responses</div></div></div>
<div class="grid" style="grid-template-columns:1fr 1fr;"><div class="card"><h3 style="margin-top:0">Add reminder</h3><label>Title</label><input [(ngModel)]="title"><label style="margin-top:.5rem">Due (ISO/local supported by browser)</label><input type="datetime-local" [(ngModel)]="due"><button class="btn" style="margin-top:.75rem" (click)="add()">Add</button>
<h3>Suggestions</h3>@for(s of suggestions();track s.application_id){<div style="padding:.4rem 0;border-bottom:1px solid var(--border)">{{s.title}}</div>}</div>
<div class="card"><h3 style="margin-top:0">All reminders</h3>@for(r of items();track r.id){<div style="display:flex;justify-content:space-between;gap:.5rem;padding:.5rem 0;border-bottom:1px solid var(--border)"><div><div>{{r.title}}</div><div class="muted">{{r.due_at|date:'medium'}} · {{r.status}}</div></div><div>@if(r.status==='pending'){<button class="btn secondary" (click)="complete(r.id)">Done</button> <button class="btn danger" (click)="dismiss(r.id)">Dismiss</button>}</div></div>}</div></div></div>` })
export class RemindersComponent implements OnInit { items=signal<any[]>([]); suggestions=signal<any[]>([]); title=''; due=''; constructor(private api:ApiService){} ngOnInit(){this.load(); this.api.get<any[]>('/reminders/suggestions').subscribe(d=>this.suggestions.set(d||[]));}
  load(){ this.api.get<any[]>('/reminders').subscribe(d=>this.items.set(d||[])); }
  add(){ if(!this.title||!this.due) return; this.api.post('/reminders',{title:this.title,due_at:new Date(this.due).toISOString()}).subscribe(()=>{this.title='';this.due='';this.load();}); }
  complete(id:string){ this.api.patch(`/reminders/${id}/complete`,{}).subscribe(()=>this.load()); }
  dismiss(id:string){ this.api.patch(`/reminders/${id}/dismiss`,{}).subscribe(()=>this.load()); }
}
