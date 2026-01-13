//! Application State and Logic
//!
//! This module defines the core application state and business logic for the
//! Converge TUI. It manages:
//!
//! - Application state (jobs, contexts, agents)
//! - User input handling and navigation
//! - Job submission and monitoring
//! - View management and transitions

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::CrosstermBackend, widgets::{ListState, TableState}, Terminal};
use std::io::Stdout;
use std::time::Duration;

pub type AppResult<T> = Result<T>;

/// Available views in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Jobs,
    JobDetail,
    Packs,
    Submit,
    Context,
    Agents,
}

impl View {
    /// Get all tab-navigable views (flat navigation)
    pub fn all() -> Vec<View> {
        vec![
            View::Jobs,
            View::Packs,
            View::Submit,
            View::Context,
            View::Agents,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            View::Jobs => "Jobs",
            View::JobDetail => "Job Details",
            View::Packs => "Packs",
            View::Submit => "Submit",
            View::Context => "Context",
            View::Agents => "Agents",
        }
    }
}

/// Breadcrumb segment for hierarchical navigation
#[derive(Debug, Clone)]
pub struct BreadcrumbSegment {
    pub label: String,
    pub view: View,
    pub data_id: Option<String>,
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Converged,
    Failed,
    Paused,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "Pending",
            JobStatus::Running => "Running",
            JobStatus::Converged => "Converged",
            JobStatus::Failed => "Failed",
            JobStatus::Paused => "Paused",
        }
    }
}

/// Job information
#[derive(Debug, Clone)]
pub struct JobInfo {
    pub id: String,
    pub pack: String,
    pub status: JobStatus,
    pub cycles: u32,
    pub facts: usize,
    pub created_at: String,
}

/// Pack information
#[derive(Debug, Clone)]
pub struct PackInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub agents: Vec<String>,
    pub invariants: Vec<String>,
}

/// Agent information
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub status: String,
    pub last_run: Option<String>,
    pub facts_produced: usize,
}

/// Fact information
#[derive(Debug, Clone)]
pub struct FactInfo {
    pub key: String,
    pub id: String,
    pub content: String,
    pub confidence: f64,
}

/// Job detail with full context
#[derive(Debug, Clone)]
pub struct JobDetail {
    pub info: JobInfo,
    pub facts: Vec<FactInfo>,
    pub agents: Vec<AgentInfo>,
    pub proposals: Vec<ProposalInfo>,
}

/// Proposal awaiting review
#[derive(Debug, Clone)]
pub struct ProposalInfo {
    pub id: String,
    pub agent: String,
    pub key: String,
    pub content: String,
    pub confidence: f64,
}

/// Submit job form
#[derive(Debug, Clone, Default)]
pub struct SubmitForm {
    pub pack: String,
    pub seeds: String,
    pub max_cycles: String,
    pub selected_field: usize,
    pub error: Option<String>,
    pub success: Option<String>,
}

impl SubmitForm {
    pub fn new() -> Self {
        Self {
            pack: String::new(),
            seeds: String::new(),
            max_cycles: "50".to_string(),
            selected_field: 0,
            error: None,
            success: None,
        }
    }
}

/// Main application state
pub struct App {
    pub running: bool,
    pub current_view: View,
    pub breadcrumb: Vec<BreadcrumbSegment>,

    // Jobs view
    pub jobs: Vec<JobInfo>,
    pub job_state: TableState,
    pub job_detail: Option<JobDetail>,

    // Packs view
    pub packs: Vec<PackInfo>,
    pub pack_state: ListState,

    // Submit view
    pub submit_form: SubmitForm,

    // Context view
    pub context_facts: Vec<FactInfo>,
    pub fact_state: ListState,

    // Agents view
    pub agents: Vec<AgentInfo>,
    pub agent_state: TableState,

    // Status
    pub status_message: Option<String>,
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        let mut job_state = TableState::default();
        job_state.select(Some(0));

        let mut pack_state = ListState::default();
        pack_state.select(Some(0));

        let mut fact_state = ListState::default();
        fact_state.select(Some(0));

        let mut agent_state = TableState::default();
        agent_state.select(Some(0));

        let mut app = Self {
            running: true,
            current_view: View::Jobs,
            breadcrumb: Vec::new(),
            jobs: Vec::new(),
            job_state,
            job_detail: None,
            packs: Vec::new(),
            pack_state,
            submit_form: SubmitForm::new(),
            context_facts: Vec::new(),
            fact_state,
            agents: Vec::new(),
            agent_state,
            status_message: None,
            loading: false,
        };
        app.update_breadcrumb();
        app.load_demo_data();
        app
    }

    /// Load demo data for testing the UI
    fn load_demo_data(&mut self) {
        // Demo jobs
        self.jobs = vec![
            JobInfo {
                id: "job-001".to_string(),
                pack: "growth-strategy".to_string(),
                status: JobStatus::Converged,
                cycles: 12,
                facts: 47,
                created_at: "2025-01-13 10:30".to_string(),
            },
            JobInfo {
                id: "job-002".to_string(),
                pack: "growth-strategy".to_string(),
                status: JobStatus::Running,
                cycles: 5,
                facts: 23,
                created_at: "2025-01-13 11:45".to_string(),
            },
            JobInfo {
                id: "job-003".to_string(),
                pack: "sdr-pipeline".to_string(),
                status: JobStatus::Paused,
                cycles: 8,
                facts: 31,
                created_at: "2025-01-13 09:15".to_string(),
            },
        ];

        // Demo packs
        self.packs = vec![
            PackInfo {
                name: "growth-strategy".to_string(),
                version: "1.0.0".to_string(),
                description: "AI-powered growth strategy generation".to_string(),
                agents: vec![
                    "MarketSignalAgent".to_string(),
                    "CompetitorAgent".to_string(),
                    "StrategyAgent".to_string(),
                    "EvaluationAgent".to_string(),
                ],
                invariants: vec![
                    "BrandSafetyInvariant".to_string(),
                    "RequireMultipleStrategies".to_string(),
                ],
            },
            PackInfo {
                name: "sdr-pipeline".to_string(),
                version: "0.5.0".to_string(),
                description: "Sales development representative automation".to_string(),
                agents: vec![
                    "LeadScoringAgent".to_string(),
                    "OutreachAgent".to_string(),
                    "FollowUpAgent".to_string(),
                ],
                invariants: vec!["ComplianceInvariant".to_string()],
            },
        ];

        // Demo agents
        self.agents = vec![
            AgentInfo {
                name: "MarketSignalAgent".to_string(),
                status: "Ready".to_string(),
                last_run: Some("2 min ago".to_string()),
                facts_produced: 5,
            },
            AgentInfo {
                name: "CompetitorAgent".to_string(),
                status: "Ready".to_string(),
                last_run: Some("2 min ago".to_string()),
                facts_produced: 3,
            },
            AgentInfo {
                name: "StrategyAgent".to_string(),
                status: "Running".to_string(),
                last_run: None,
                facts_produced: 0,
            },
        ];

        // Demo context facts
        self.context_facts = vec![
            FactInfo {
                key: "Seeds".to_string(),
                id: "seed-001".to_string(),
                content: "Target market: Nordic region, B2B SaaS".to_string(),
                confidence: 1.0,
            },
            FactInfo {
                key: "MarketSignals".to_string(),
                id: "signal-001".to_string(),
                content: "Growing demand for AI automation in Nordic enterprises".to_string(),
                confidence: 0.85,
            },
            FactInfo {
                key: "Competitors".to_string(),
                id: "comp-001".to_string(),
                content: "Main competitor: Acme Corp, 30% market share".to_string(),
                confidence: 0.92,
            },
        ];
    }

    /// Update breadcrumb based on current view
    pub fn update_breadcrumb(&mut self) {
        self.breadcrumb.clear();

        match self.current_view {
            View::Jobs => {
                self.breadcrumb.push(BreadcrumbSegment {
                    label: "Jobs".to_string(),
                    view: View::Jobs,
                    data_id: None,
                });
            }
            View::JobDetail => {
                self.breadcrumb.push(BreadcrumbSegment {
                    label: "Jobs".to_string(),
                    view: View::Jobs,
                    data_id: None,
                });
                if let Some(ref detail) = self.job_detail {
                    self.breadcrumb.push(BreadcrumbSegment {
                        label: detail.info.id.clone(),
                        view: View::JobDetail,
                        data_id: Some(detail.info.id.clone()),
                    });
                }
            }
            View::Packs => {
                self.breadcrumb.push(BreadcrumbSegment {
                    label: "Packs".to_string(),
                    view: View::Packs,
                    data_id: None,
                });
            }
            View::Submit => {
                self.breadcrumb.push(BreadcrumbSegment {
                    label: "Submit".to_string(),
                    view: View::Submit,
                    data_id: None,
                });
            }
            View::Context => {
                self.breadcrumb.push(BreadcrumbSegment {
                    label: "Context".to_string(),
                    view: View::Context,
                    data_id: None,
                });
            }
            View::Agents => {
                self.breadcrumb.push(BreadcrumbSegment {
                    label: "Agents".to_string(),
                    view: View::Agents,
                    data_id: None,
                });
            }
        }
    }

    /// Navigate to next tab view
    pub fn next_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| *v == self.current_view).unwrap_or(0);
        let next_idx = (current_idx + 1) % views.len();
        self.current_view = views[next_idx];
        self.update_breadcrumb();
    }

    /// Navigate to previous tab view
    pub fn prev_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| *v == self.current_view).unwrap_or(0);
        let prev_idx = if current_idx == 0 { views.len() - 1 } else { current_idx - 1 };
        self.current_view = views[prev_idx];
        self.update_breadcrumb();
    }

    /// Go to specific tab view by index
    pub fn goto_view(&mut self, index: usize) {
        let views = View::all();
        if index < views.len() {
            self.current_view = views[index];
            self.update_breadcrumb();
        }
    }

    /// Navigate back via breadcrumb
    pub fn navigate_back(&mut self) {
        if self.breadcrumb.len() > 1 {
            self.breadcrumb.pop();
            if let Some(segment) = self.breadcrumb.last() {
                self.current_view = segment.view;
            }
        }
    }

    /// Select next item in current list
    pub fn select_next(&mut self) {
        match self.current_view {
            View::Jobs => {
                let len = self.jobs.len();
                if len > 0 {
                    let i = self.job_state.selected().unwrap_or(0);
                    self.job_state.select(Some((i + 1) % len));
                }
            }
            View::Packs => {
                let len = self.packs.len();
                if len > 0 {
                    let i = self.pack_state.selected().unwrap_or(0);
                    self.pack_state.select(Some((i + 1) % len));
                }
            }
            View::Context => {
                let len = self.context_facts.len();
                if len > 0 {
                    let i = self.fact_state.selected().unwrap_or(0);
                    self.fact_state.select(Some((i + 1) % len));
                }
            }
            View::Agents => {
                let len = self.agents.len();
                if len > 0 {
                    let i = self.agent_state.selected().unwrap_or(0);
                    self.agent_state.select(Some((i + 1) % len));
                }
            }
            View::Submit => {
                self.submit_form.selected_field = (self.submit_form.selected_field + 1) % 3;
            }
            _ => {}
        }
    }

    /// Select previous item in current list
    pub fn select_prev(&mut self) {
        match self.current_view {
            View::Jobs => {
                let len = self.jobs.len();
                if len > 0 {
                    let i = self.job_state.selected().unwrap_or(0);
                    self.job_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
            View::Packs => {
                let len = self.packs.len();
                if len > 0 {
                    let i = self.pack_state.selected().unwrap_or(0);
                    self.pack_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
            View::Context => {
                let len = self.context_facts.len();
                if len > 0 {
                    let i = self.fact_state.selected().unwrap_or(0);
                    self.fact_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
            View::Agents => {
                let len = self.agents.len();
                if len > 0 {
                    let i = self.agent_state.selected().unwrap_or(0);
                    self.agent_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
            View::Submit => {
                self.submit_form.selected_field = if self.submit_form.selected_field == 0 { 2 } else { self.submit_form.selected_field - 1 };
            }
            _ => {}
        }
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        if self.current_view == View::Submit {
            let field = match self.submit_form.selected_field {
                0 => &mut self.submit_form.pack,
                1 => &mut self.submit_form.seeds,
                2 => &mut self.submit_form.max_cycles,
                _ => return,
            };
            field.push(c);
            self.submit_form.error = None;
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if self.current_view == View::Submit {
            let field = match self.submit_form.selected_field {
                0 => &mut self.submit_form.pack,
                1 => &mut self.submit_form.seeds,
                2 => &mut self.submit_form.max_cycles,
                _ => return,
            };
            field.pop();
        }
    }

    /// Enter job detail view
    pub fn enter_job_detail(&mut self) {
        if let Some(idx) = self.job_state.selected() {
            if let Some(job) = self.jobs.get(idx) {
                // Create demo job detail
                self.job_detail = Some(JobDetail {
                    info: job.clone(),
                    facts: self.context_facts.clone(),
                    agents: self.agents.clone(),
                    proposals: vec![
                        ProposalInfo {
                            id: "prop-001".to_string(),
                            agent: "StrategyAgent".to_string(),
                            key: "Strategies".to_string(),
                            content: "Expand to Denmark market with localized offering".to_string(),
                            confidence: 0.78,
                        },
                    ],
                });
                self.current_view = View::JobDetail;
                self.update_breadcrumb();
            }
        }
    }

    /// Submit a new job
    pub fn submit_job(&mut self) {
        if self.submit_form.pack.is_empty() {
            self.submit_form.error = Some("Pack name is required".to_string());
            return;
        }

        // Validate pack exists
        if !self.packs.iter().any(|p| p.name == self.submit_form.pack) {
            self.submit_form.error = Some(format!("Pack '{}' not found", self.submit_form.pack));
            return;
        }

        // Create new job
        let job_id = format!("job-{:03}", self.jobs.len() + 1);
        self.jobs.insert(0, JobInfo {
            id: job_id.clone(),
            pack: self.submit_form.pack.clone(),
            status: JobStatus::Pending,
            cycles: 0,
            facts: 0,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        });

        self.submit_form.success = Some(format!("Job {} submitted successfully", job_id));
        self.submit_form.pack.clear();
        self.submit_form.seeds.clear();
        self.submit_form.max_cycles = "50".to_string();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Main event loop
pub async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
) -> AppResult<()> {
    loop {
        terminal.draw(|f| super::views::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        // Quit
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if app.current_view == View::JobDetail {
                                app.navigate_back();
                            } else if app.current_view == View::Submit && !app.submit_form.pack.is_empty() {
                                // Clear form on first Esc, quit on second
                                app.submit_form = SubmitForm::new();
                            } else {
                                app.running = false;
                            }
                        }
                        // Tab navigation
                        KeyCode::Tab => {
                            app.next_view();
                        }
                        KeyCode::BackTab => {
                            app.prev_view();
                        }
                        KeyCode::Right => {
                            app.next_view();
                        }
                        KeyCode::Left => {
                            if app.current_view == View::JobDetail {
                                app.navigate_back();
                            } else {
                                app.prev_view();
                            }
                        }
                        // Direct tab access with Ctrl+Number
                        KeyCode::Char('1') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.goto_view(0);
                        }
                        KeyCode::Char('2') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.goto_view(1);
                        }
                        KeyCode::Char('3') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.goto_view(2);
                        }
                        KeyCode::Char('4') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.goto_view(3);
                        }
                        KeyCode::Char('5') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.goto_view(4);
                        }
                        // List navigation
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.select_next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.select_prev();
                        }
                        // Enter actions
                        KeyCode::Enter => {
                            match app.current_view {
                                View::Jobs => {
                                    app.enter_job_detail();
                                }
                                View::Submit => {
                                    if app.submit_form.selected_field == 2 {
                                        app.submit_job();
                                    } else {
                                        app.submit_form.selected_field += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        // Back navigation
                        KeyCode::Char('b') => {
                            if app.breadcrumb.len() > 1 {
                                app.navigate_back();
                            }
                        }
                        // Text input
                        KeyCode::Char(c) => {
                            app.handle_char(c);
                        }
                        KeyCode::Backspace => {
                            app.handle_backspace();
                        }
                        _ => {}
                    }
                }
            }
        }

        if !app.running {
            return Ok(());
        }
    }
}
