-- Rename the misleading `score_utilization` column to what it always held: the
-- scheduled-ratio metric (shown as "排期覆盖" in the UI). RENAME COLUMN preserves
-- all existing row data (SQLite ≥ 3.25). `score_fairness` keeps its name — it now
-- stores a real Jain index instead of a constant 0.0, but the column name already
-- matches its content.
ALTER TABLE ai_optimization_runs RENAME COLUMN score_utilization TO score_scheduled_ratio;
