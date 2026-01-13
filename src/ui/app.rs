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
use std::sync::Arc;
use std::time::Duration;

use converge_core::{Context, ContextKey, Engine, Fact};
use converge_core::llm::LlmProvider;
use converge_provider::{AnthropicProvider, OpenAiProvider};
use strum::IntoEnumIterator;

use crate::agents::{MockInsightProvider, RiskAssessmentAgent, StrategicInsightAgent};
use crate::packs;
use converge_domain::growth_strategy::{
    BrandSafetyInvariant, CompetitorAgent, EvaluationAgent, MarketSignalAgent,
    RequireEvaluationRationale, RequireMultipleStrategies, RequireStrategyEvaluations,
    StrategyAgent,
};

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
    pub job_details_cache: std::collections::HashMap<String, JobDetail>,

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
            job_details_cache: std::collections::HashMap::new(),
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

    /// Load real pack data from the packs module
    fn load_demo_data(&mut self) {
        // Load real packs from the packs module
        let available = packs::available_packs();
        self.packs = available
            .iter()
            .map(|name| {
                let info = packs::pack_info(name);
                PackInfo {
                    name: info.name,
                    version: info.version,
                    description: info.description,
                    agents: get_pack_agents(name),
                    invariants: info.invariants,
                }
            })
            .collect();

        // Initialize agents list for growth-strategy (default pack)
        self.agents = vec![
            AgentInfo {
                name: "MarketSignalAgent".to_string(),
                status: "Ready".to_string(),
                last_run: None,
                facts_produced: 0,
            },
            AgentInfo {
                name: "CompetitorAgent".to_string(),
                status: "Ready".to_string(),
                last_run: None,
                facts_produced: 0,
            },
            AgentInfo {
                name: "StrategyAgent".to_string(),
                status: "Ready".to_string(),
                last_run: None,
                facts_produced: 0,
            },
            AgentInfo {
                name: "EvaluationAgent".to_string(),
                status: "Ready".to_string(),
                last_run: None,
                facts_produced: 0,
            },
            AgentInfo {
                name: "StrategicInsightAgent".to_string(),
                status: "Ready".to_string(),
                last_run: None,
                facts_produced: 0,
            },
            AgentInfo {
                name: "RiskAssessmentAgent".to_string(),
                status: "Ready".to_string(),
                last_run: None,
                facts_produced: 0,
            },
        ];

        // Start with empty jobs (no demo jobs)
        self.jobs = Vec::new();

        // Empty context facts until a job is run
        self.context_facts = Vec::new();
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
                // Try to get cached job detail
                if let Some(detail) = self.job_details_cache.get(&job.id) {
                    self.job_detail = Some(detail.clone());
                    // Update context facts to show this job's facts
                    self.context_facts = detail.facts.clone();
                } else {
                    // No cached detail - create a minimal one
                    self.job_detail = Some(JobDetail {
                        info: job.clone(),
                        facts: Vec::new(),
                        agents: self.agents.clone(),
                        proposals: Vec::new(),
                    });
                }
                self.current_view = View::JobDetail;
                self.update_breadcrumb();
            }
        }
    }

    /// Submit and run a new job using the real convergence engine
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

        let job_id = format!("job-{:03}", self.jobs.len() + 1);
        let pack_name = self.submit_form.pack.clone();
        let seeds_json = self.submit_form.seeds.clone();

        // Parse seeds if provided
        let mut context = Context::new();
        if !seeds_json.is_empty() {
            match serde_json::from_str::<Vec<converge_runtime::templates::SeedFact>>(&seeds_json) {
                Ok(seed_facts) => {
                    for seed in seed_facts {
                        let fact = Fact::new(ContextKey::Seeds, seed.id, seed.content);
                        if let Err(e) = context.add_fact(fact) {
                            self.submit_form.error = Some(format!("Failed to add seed: {}", e));
                            return;
                        }
                    }
                }
                Err(e) => {
                    self.submit_form.error = Some(format!("Invalid seeds JSON: {}", e));
                    return;
                }
            }
        }

        // Run convergence engine
        let mut engine = Engine::new();

        // Register agents for the pack
        if let Err(e) = register_pack_agents(&mut engine, &pack_name) {
            self.submit_form.error = Some(format!("Failed to register agents: {}", e));
            return;
        }

        // Run the convergence loop
        match engine.run(context) {
            Ok(result) => {
                // Calculate total facts
                let total_facts: usize = ContextKey::iter()
                    .map(|key| result.context.get(key).len())
                    .sum();

                let status = if result.converged {
                    JobStatus::Converged
                } else {
                    JobStatus::Failed
                };

                // Convert facts to FactInfo
                let facts: Vec<FactInfo> = ContextKey::iter()
                    .flat_map(|key| {
                        result.context.get(key).iter().map(|fact| {
                            FactInfo {
                                key: format!("{:?}", fact.key),
                                id: fact.id.clone(),
                                content: fact.content.clone(),
                                confidence: 1.0,
                            }
                        }).collect::<Vec<_>>()
                    })
                    .collect();

                // Update context facts for the Context view
                self.context_facts = facts.clone();

                // Create job info
                let job = JobInfo {
                    id: job_id.clone(),
                    pack: pack_name.clone(),
                    status,
                    cycles: result.cycles,
                    facts: total_facts,
                    created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                };

                // Create job detail
                let detail = JobDetail {
                    info: job.clone(),
                    facts: facts.clone(),
                    agents: self.agents.clone(),
                    proposals: Vec::new(),
                };

                // Store job and detail
                self.job_details_cache.insert(job_id.clone(), detail.clone());
                self.job_detail = Some(detail);
                self.jobs.insert(0, job);

                let status_msg = if result.converged {
                    format!("Job {} converged in {} cycles with {} facts", job_id, result.cycles, total_facts)
                } else {
                    format!("Job {} halted after {} cycles with {} facts", job_id, result.cycles, total_facts)
                };
                self.submit_form.success = Some(status_msg);
            }
            Err(e) => {
                // Create failed job entry
                self.jobs.insert(0, JobInfo {
                    id: job_id.clone(),
                    pack: pack_name,
                    status: JobStatus::Failed,
                    cycles: 0,
                    facts: 0,
                    created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                });
                self.submit_form.error = Some(format!("Job failed: {}", e));
            }
        }

        // Clear form
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

/// Get the list of agents for a pack
fn get_pack_agents(pack_name: &str) -> Vec<String> {
    match pack_name {
        "growth-strategy" => vec![
            "MarketSignalAgent".to_string(),
            "CompetitorAgent".to_string(),
            "StrategyAgent".to_string(),
            "EvaluationAgent".to_string(),
            "StrategicInsightAgent".to_string(),
            "RiskAssessmentAgent".to_string(),
        ],
        "sdr-pipeline" => vec![
            "LeadScoringAgent".to_string(),
            "OutreachAgent".to_string(),
            "FollowUpAgent".to_string(),
        ],
        _ => Vec::new(),
    }
}

/// Creates an LLM provider from environment variables.
fn create_llm_provider() -> Arc<dyn LlmProvider> {
    tokio::task::block_in_place(|| {
        // Try Anthropic first
        if let Ok(provider) = AnthropicProvider::from_env("claude-sonnet-4-20250514") {
            return Arc::new(provider) as Arc<dyn LlmProvider>;
        }

        // Try OpenAI second
        if let Ok(provider) = OpenAiProvider::from_env("gpt-4o") {
            return Arc::new(provider) as Arc<dyn LlmProvider>;
        }

        // Fall back to mock provider
        Arc::new(MockInsightProvider::default_insights()) as Arc<dyn LlmProvider>
    })
}

/// Register agents and invariants for a specific domain pack.
fn register_pack_agents(engine: &mut Engine, pack_name: &str) -> Result<()> {
    match pack_name {
        "growth-strategy" => {
            // Register deterministic agents
            engine.register(MarketSignalAgent);
            engine.register(CompetitorAgent);
            engine.register(StrategyAgent);
            engine.register(EvaluationAgent);

            // Create LLM provider (shared by all LLM agents)
            let llm_provider = create_llm_provider();

            // Register LLM-powered agents
            engine.register(StrategicInsightAgent::new(llm_provider.clone()));
            engine.register(RiskAssessmentAgent::new(llm_provider));

            // Register Invariants
            engine.register_invariant(BrandSafetyInvariant::default());
            engine.register_invariant(RequireMultipleStrategies);
            engine.register_invariant(RequireStrategyEvaluations);
            engine.register_invariant(RequireEvaluationRationale);
        }
        _ => {
            return Err(anyhow::anyhow!("Pack '{}' not implemented", pack_name));
        }
    }
    Ok(())
}
