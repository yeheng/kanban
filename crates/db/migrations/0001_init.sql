-- 0001_init.sql — full schema (design §3.3)

CREATE TABLE settings (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    default_unit        TEXT    NOT NULL DEFAULT 'PD'  CHECK (default_unit IN ('PD','PM')),
    pd_hours            REAL    NOT NULL DEFAULT 8.0    CHECK (pd_hours > 0),
    pm_workdays         REAL    NOT NULL DEFAULT 20.0   CHECK (pm_workdays > 0),
    ai_provider         TEXT    NOT NULL DEFAULT 'ollama',
    ai_base_url         TEXT,
    ai_api_key_enc      TEXT,
    secret_store        TEXT    NOT NULL DEFAULT 'keychain' CHECK (secret_store IN ('keychain','encrypted_file')),
    ai_chat_model       TEXT    NOT NULL DEFAULT 'qwen2.5:7b',
    ai_embed_model      TEXT    NOT NULL DEFAULT 'nomic-embed-text',
    ai_embed_dim        INTEGER NOT NULL DEFAULT 768,
    solver_backend      TEXT    NOT NULL DEFAULT 'good_lp' CHECK (solver_backend IN ('good_lp','greedy','hungarian')),
    solver_timeout_ms   INTEGER NOT NULL DEFAULT 5000,
    locale              TEXT    NOT NULL DEFAULT 'zh-CN',
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
INSERT INTO settings (id) VALUES (1);

CREATE TABLE tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    color       TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_tags_name ON tags(name);

CREATE TABLE skills (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_skills_name ON skills(name);

CREATE TABLE resources (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    name                        TEXT    NOT NULL,
    email                       TEXT,
    available_from              TEXT,
    available_to                TEXT,
    status                      TEXT    NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive','archived')),
    daily_capacity_pd           REAL    NOT NULL DEFAULT 1.0 CHECK (daily_capacity_pd > 0),
    daily_rate_pd               REAL,
    max_parallel_tasks_per_day  INTEGER,
    metadata                    TEXT,
    created_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at                  TEXT
);
CREATE INDEX idx_resources_status ON resources(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_name ON resources(name);

CREATE TABLE resource_skills (
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    skill_id        INTEGER NOT NULL REFERENCES skills(id)    ON DELETE CASCADE,
    proficiency     INTEGER NOT NULL CHECK (proficiency BETWEEN 1 AND 5),
    evidence        TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (resource_id, skill_id)
);
CREATE INDEX idx_resource_skills_skill ON resource_skills(skill_id);

CREATE TABLE resource_tags (
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id)      ON DELETE CASCADE,
    PRIMARY KEY (resource_id, tag_id)
);
CREATE INDEX idx_resource_tags_tag ON resource_tags(tag_id);

CREATE TABLE teams (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at  TEXT
);
CREATE UNIQUE INDEX idx_teams_name_active ON teams(name) WHERE deleted_at IS NULL;

CREATE TABLE team_members (
    team_id     INTEGER NOT NULL REFERENCES teams(id)     ON DELETE CASCADE,
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    role        TEXT,
    joined_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (team_id, resource_id)
);
CREATE INDEX idx_team_members_resource ON team_members(resource_id);

CREATE TABLE team_overrides (
    team_id             INTEGER PRIMARY KEY REFERENCES teams(id) ON DELETE CASCADE,
    pd_hours            REAL    CHECK (pd_hours IS NULL OR pd_hours > 0),
    pm_workdays         REAL    CHECK (pm_workdays IS NULL OR pm_workdays > 0),
    overload_threshold  REAL    CHECK (overload_threshold IS NULL OR overload_threshold > 0),
    underload_threshold REAL    CHECK (underload_threshold IS NULL OR underload_threshold >= 0),
    utilization_green   REAL    CHECK (utilization_green IS NULL OR (utilization_green >= 0 AND utilization_green <= 1.0)),
    utilization_yellow  REAL    CHECK (utilization_yellow IS NULL OR (utilization_yellow >= 0 AND utilization_yellow <= 1.0)),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- NOTE: projects + tasks created here (before work_week_template / holiday) so their
-- forward FK references resolve cleanly under foreign_keys=ON at migration time.

CREATE TABLE projects (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    name                        TEXT    NOT NULL,
    description                 TEXT,
    start_date                  TEXT,
    end_date                    TEXT,
    priority                    INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 1 AND 9),
    budget_pd                   REAL    NOT NULL DEFAULT 0 CHECK (budget_pd >= 0),
    max_parallel_tasks_per_day  INTEGER,
    status                      TEXT    NOT NULL DEFAULT 'planning' CHECK (status IN ('planning','active','on_hold','done','cancelled')),
    metadata                    TEXT,
    created_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at                  TEXT,
    CHECK (end_date IS NULL OR start_date IS NULL OR end_date >= start_date)
);
CREATE INDEX idx_projects_status ON projects(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_projects_dates ON projects(start_date, end_date) WHERE deleted_at IS NULL;

CREATE TABLE tasks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_task_id  INTEGER REFERENCES tasks(id) ON DELETE CASCADE,
    title           TEXT    NOT NULL,
    description     TEXT,
    estimate_pd     REAL    NOT NULL DEFAULT 0 CHECK (estimate_pd >= 0),
    start_date      TEXT,
    end_date        TEXT,
    is_long_term    INTEGER NOT NULL DEFAULT 0 CHECK (is_long_term IN (0,1)),
    segment_kind    TEXT    CHECK (segment_kind IN ('milestone','phase','segment') OR segment_kind IS NULL),
    status          TEXT    NOT NULL DEFAULT 'todo' CHECK (status IN ('todo','in_progress','blocked','review','done','cancelled')),
    sort_order      INTEGER NOT NULL DEFAULT 0,
    metadata        TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at      TEXT,
    CHECK (end_date IS NULL OR start_date IS NULL OR end_date >= start_date)
);
CREATE INDEX idx_tasks_project ON tasks(project_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_parent  ON tasks(parent_task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_status  ON tasks(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_dates   ON tasks(start_date, end_date) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_longterm ON tasks(is_long_term) WHERE is_long_term = 1 AND deleted_at IS NULL;

CREATE TABLE work_week_template (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    scope       TEXT    NOT NULL CHECK (scope IN ('global','project')),
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    mon         INTEGER NOT NULL DEFAULT 1 CHECK (mon IN (0,1)),
    tue         INTEGER NOT NULL DEFAULT 1 CHECK (tue IN (0,1)),
    wed         INTEGER NOT NULL DEFAULT 1 CHECK (wed IN (0,1)),
    thu         INTEGER NOT NULL DEFAULT 1 CHECK (thu IN (0,1)),
    fri         INTEGER NOT NULL DEFAULT 1 CHECK (fri IN (0,1)),
    sat         INTEGER NOT NULL DEFAULT 0 CHECK (sat IN (0,1)),
    sun         INTEGER NOT NULL DEFAULT 0 CHECK (sun IN (0,1)),
    mon_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (mon_frac > 0 AND mon_frac <= 1.0),
    tue_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (tue_frac > 0 AND tue_frac <= 1.0),
    wed_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (wed_frac > 0 AND wed_frac <= 1.0),
    thu_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (thu_frac > 0 AND thu_frac <= 1.0),
    fri_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (fri_frac > 0 AND fri_frac <= 1.0),
    sat_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (sat_frac > 0 AND sat_frac <= 1.0),
    sun_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (sun_frac > 0 AND sun_frac <= 1.0),
    CHECK ((scope = 'global' AND project_id IS NULL) OR (scope = 'project' AND project_id IS NOT NULL))
);
CREATE UNIQUE INDEX idx_wwt_global ON work_week_template((1)) WHERE scope='global';
CREATE UNIQUE INDEX idx_wwt_project ON work_week_template(project_id) WHERE scope='project';
INSERT OR IGNORE INTO work_week_template (scope, mon,tue,wed,thu,fri,sat,sun) VALUES ('global', 1,1,1,1,1,0,0);

CREATE TABLE holiday (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    day         TEXT    NOT NULL,
    fraction    REAL    NOT NULL DEFAULT 1.0 CHECK (fraction > 0 AND fraction <= 1.0),
    name        TEXT,
    CHECK (length(day) = 10)
);
CREATE INDEX idx_holiday_day ON holiday(day);
CREATE INDEX idx_holiday_project_day ON holiday(project_id, day);

CREATE TABLE time_off (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    day         TEXT    NOT NULL,
    fraction    REAL    NOT NULL DEFAULT 1.0 CHECK (fraction > 0 AND fraction <= 1.0),
    reason      TEXT,
    note        TEXT,
    CHECK (length(day) = 10)
);
CREATE INDEX idx_time_off_res_day ON time_off(resource_id, day);

CREATE TABLE task_dependencies (
    task_id         INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    predecessor_id  INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    lag_days        INTEGER NOT NULL DEFAULT 0,
    dep_type        TEXT    NOT NULL DEFAULT 'FS' CHECK (dep_type IN ('FS','FF','SS','SF')),
    PRIMARY KEY (task_id, predecessor_id),
    CHECK (task_id <> predecessor_id)
);
CREATE INDEX idx_deps_predecessor ON task_dependencies(predecessor_id);

CREATE TABLE task_skill_requirements (
    task_id             INTEGER NOT NULL REFERENCES tasks(id)  ON DELETE CASCADE,
    skill_id            INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    min_proficiency     INTEGER NOT NULL CHECK (min_proficiency BETWEEN 1 AND 5),
    is_mandatory        INTEGER NOT NULL DEFAULT 1 CHECK (is_mandatory IN (0,1)),
    weight              REAL    NOT NULL DEFAULT 1.0 CHECK (weight >= 0),
    PRIMARY KEY (task_id, skill_id)
);
CREATE INDEX idx_task_req_skill ON task_skill_requirements(skill_id);

CREATE TABLE task_tags (
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);
CREATE INDEX idx_task_tags_tag ON task_tags(tag_id);

CREATE TABLE ai_optimization_runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    seed                INTEGER NOT NULL,
    objective           TEXT    NOT NULL DEFAULT 'balanced' CHECK (objective IN ('balanced','min_makespan','max_utilization','fairness','skill_fit')),
    scope               TEXT    NOT NULL DEFAULT 'full' CHECK (scope IN ('full','incremental')),
    scope_project_ids   TEXT,
    scope_from          TEXT,
    scope_to            TEXT,
    config_json         TEXT NOT NULL,
    constraints_json    TEXT NOT NULL,
    weights_json        TEXT NOT NULL,
    input_snapshot_json TEXT NOT NULL,
    output_plan_json    TEXT,
    score_overall       REAL,
    score_skill_fit     REAL,
    score_utilization   REAL,
    score_fairness      REAL,
    explanation_md      TEXT,
    provider            TEXT NOT NULL,
    chat_model          TEXT NOT NULL,
    embed_model         TEXT,
    solver_backend      TEXT NOT NULL,
    solver_status       TEXT NOT NULL CHECK (solver_status IN ('optimal','feasible','infeasible','timeout','error')),
    status              TEXT    NOT NULL DEFAULT 'proposed' CHECK (status IN ('proposed','accepted','rejected')),
    applied             INTEGER NOT NULL DEFAULT 0 CHECK (applied IN (0,1)),
    started_at          TEXT NOT NULL,
    finished_at         TEXT,
    duration_ms         INTEGER,
    error_msg           TEXT,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_runs_applied ON ai_optimization_runs(applied, created_at);
CREATE INDEX idx_runs_scope ON ai_optimization_runs(scope_from, scope_to);
CREATE INDEX idx_runs_status ON ai_optimization_runs(status, created_at);

CREATE TABLE allocations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    task_id         INTEGER NOT NULL REFERENCES tasks(id)     ON DELETE CASCADE,
    start_date      TEXT    NOT NULL,
    end_date        TEXT    NOT NULL,
    percent         REAL    NOT NULL CHECK (percent > 0 AND percent <= 1.0),
    allocated_pd    REAL    NOT NULL DEFAULT 0 CHECK (allocated_pd >= 0),
    status          TEXT    NOT NULL DEFAULT 'planned' CHECK (status IN ('planned','committed','in_progress','done','cancelled','locked')),
    source          TEXT    NOT NULL DEFAULT 'manual' CHECK (source IN ('manual','ai')),
    run_id          INTEGER REFERENCES ai_optimization_runs(id) ON DELETE SET NULL,
    note            TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at      TEXT,
    CHECK (end_date >= start_date),
    CHECK (source <> 'ai' OR run_id IS NOT NULL)
);
CREATE INDEX idx_alloc_resource_date ON allocations(resource_id, start_date, end_date) WHERE deleted_at IS NULL;
CREATE INDEX idx_alloc_task ON allocations(task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_alloc_run ON allocations(run_id) WHERE run_id IS NOT NULL;
CREATE INDEX idx_alloc_status ON allocations(status) WHERE deleted_at IS NULL;

CREATE TRIGGER trg_allocation_validate_insert
AFTER INSERT ON allocations
BEGIN
    SELECT RAISE(ABORT, 'allocation out of task window')
    FROM tasks t
    WHERE t.id = NEW.task_id
      AND t.start_date IS NOT NULL AND t.end_date IS NOT NULL
      AND (NEW.start_date < t.start_date OR NEW.end_date > t.end_date);
    SELECT RAISE(ABORT, 'allocation out of resource availability')
    FROM resources r
    WHERE r.id = NEW.resource_id
      AND r.available_from IS NOT NULL AND r.available_to IS NOT NULL
      AND (NEW.start_date < r.available_from OR NEW.end_date > r.available_to);
    SELECT RAISE(ABORT, 'allocation.percent invalid')
    WHERE NEW.percent <= 0 OR NEW.percent > 1.0;
END;

CREATE TRIGGER trg_allocation_validate_update
AFTER UPDATE OF start_date, end_date, resource_id, task_id, percent ON allocations
BEGIN
    SELECT RAISE(ABORT, 'allocation out of task window')
    FROM tasks t
    WHERE t.id = NEW.task_id
      AND t.start_date IS NOT NULL AND t.end_date IS NOT NULL
      AND (NEW.start_date < t.start_date OR NEW.end_date > t.end_date);
    SELECT RAISE(ABORT, 'allocation out of resource availability')
    FROM resources r
    WHERE r.id = NEW.resource_id
      AND r.available_from IS NOT NULL AND r.available_to IS NOT NULL
      AND (NEW.start_date < r.available_from OR NEW.end_date > r.available_to);
    SELECT RAISE(ABORT, 'allocation.percent invalid')
    WHERE NEW.percent <= 0 OR NEW.percent > 1.0;
END;

CREATE TABLE resource_project_rates (
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    project_id      INTEGER NOT NULL REFERENCES projects(id)  ON DELETE CASCADE,
    daily_rate_pd   REAL    NOT NULL CHECK (daily_rate_pd > 0),
    valid_from      TEXT,
    valid_to        TEXT,
    note            TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (resource_id, project_id, valid_from),
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from)
);
CREATE INDEX idx_rpr_project ON resource_project_rates(project_id);
CREATE INDEX idx_rpr_res_date ON resource_project_rates(resource_id, valid_from, valid_to);
