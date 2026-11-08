import { Routes } from '@angular/router';
import { authGuard } from './core/guards/auth.guard';
import { ShellComponent } from './layout/shell.component';

export const routes: Routes = [
  { path: 'login', loadComponent: () => import('./features/login/login.component').then(m => m.LoginComponent) },
  { path: 'callback', loadComponent: () => import('./features/auth-callback/callback.component').then(m => m.CallbackComponent) },
  {
    path: '',
    component: ShellComponent,
    canActivate: [authGuard],
    children: [
      { path: '', pathMatch: 'full', redirectTo: 'dashboard' },
      { path: 'dashboard', loadComponent: () => import('./features/dashboard/dashboard.component').then(m => m.DashboardComponent) },
      { path: 'applications', loadComponent: () => import('./features/applications/applications.component').then(m => m.ApplicationsComponent) },
      { path: 'jobs', loadComponent: () => import('./features/jobs/jobs.component').then(m => m.JobsComponent) },
      { path: 'companies', loadComponent: () => import('./features/companies/companies.component').then(m => m.CompaniesComponent) },
      { path: 'analytics', loadComponent: () => import('./features/analytics/analytics.component').then(m => m.AnalyticsComponent) },
      { path: 'email-activity', loadComponent: () => import('./features/email-activity/email-activity.component').then(m => m.EmailActivityComponent) },
      { path: 'reminders', loadComponent: () => import('./features/reminders/reminders.component').then(m => m.RemindersComponent) },
      { path: 'settings', loadComponent: () => import('./features/settings/settings.component').then(m => m.SettingsComponent) },
    ]
  },
  { path: '**', redirectTo: '' }
];
