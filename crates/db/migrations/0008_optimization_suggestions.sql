-- 0008_optimization_suggestions.sql
-- LLM 给出的、对 solver 方案的结构化改进建议。每条建议绑定到 task/resource，
-- 是"对 problem 的修改意图"而非最终分配；采纳后经 rerun 重跑求解器才落地。
CREATE TABLE ai_optimization_suggestions (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id             INTEGER NOT NULL REFERENCES ai_optimization_runs(id) ON DELETE CASCADE,
    kind               TEXT    NOT NULL CHECK (kind IN (
                            'swap_resource','change_percent','widen_window','drop_dependency',
                            'add_resource','widen_resource_window','change_resource_capacity',
                            'upsert_resource_skill')),
    target_task_id     INTEGER,
    target_resource_id INTEGER,
    payload_json       TEXT    NOT NULL,
    rationale_md       TEXT    NOT NULL,
    status             TEXT    NOT NULL DEFAULT 'proposed'
                        CHECK (status IN ('proposed','accepted','skipped','applied')),
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_optimization_suggestions_run ON ai_optimization_suggestions(run_id);

ALTER TABLE ai_optimization_runs ADD COLUMN parent_run_id INTEGER
    REFERENCES ai_optimization_runs(id) ON DELETE SET NULL;
CREATE INDEX idx_optimization_runs_parent ON ai_optimization_runs(parent_run_id);

ALTER TABLE settings ADD COLUMN use_llm_advisor INTEGER NOT NULL DEFAULT 0 CHECK (use_llm_advisor IN (0,1));
