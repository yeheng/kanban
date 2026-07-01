-- 0005: separate embedding LLM configuration from chat LLM configuration.
--
-- Previously a single set of provider/base_url/api_key/chat/embed columns was
-- shared between the chat LLM (explainer) and the embedding LLM (semantic
-- scorer). This migration splits out a dedicated embedding config so users can
-- point chat and embedding at different providers/URLs/models.

ALTER TABLE settings ADD COLUMN embed_provider    TEXT NOT NULL DEFAULT 'ollama';
ALTER TABLE settings ADD COLUMN embed_base_url    TEXT;
ALTER TABLE settings ADD COLUMN embed_api_key_enc TEXT;
ALTER TABLE settings ADD COLUMN embed_model       TEXT NOT NULL DEFAULT 'nomic-embed-text';
ALTER TABLE settings ADD COLUMN embed_dim         INTEGER NOT NULL DEFAULT 768;

-- Migrate existing settings: embedding config initially mirrors chat config.
UPDATE settings
SET embed_provider = ai_provider,
    embed_base_url = ai_base_url,
    embed_api_key_enc = ai_api_key_enc,
    embed_model = ai_embed_model,
    embed_dim = ai_embed_dim
WHERE id = 1;

-- Drop old combined embedding columns.
ALTER TABLE settings DROP COLUMN ai_embed_model;
ALTER TABLE settings DROP COLUMN ai_embed_dim;
