-- 0006: add explicit LLM feature flags to settings.
--
-- Previously semantic scoring / LLM explanation were gated by environment variables
-- (KANBAN_USE_SEMANTIC / KANBAN_USE_LLM_EXPLAINER). This migration moves the control
-- into the database so it can be exposed and toggled from the frontend settings page.

ALTER TABLE settings ADD COLUMN use_semantic_scorer INTEGER NOT NULL DEFAULT 1 CHECK (use_semantic_scorer IN (0,1));
ALTER TABLE settings ADD COLUMN use_llm_explainer  INTEGER NOT NULL DEFAULT 1 CHECK (use_llm_explainer IN (0,1));

UPDATE settings SET use_semantic_scorer = 1, use_llm_explainer = 1 WHERE id = 1;
