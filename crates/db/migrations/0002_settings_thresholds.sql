-- 0002: global utilization thresholds (team-level overrides live in team_overrides).
ALTER TABLE settings ADD COLUMN overload_threshold  REAL CHECK (overload_threshold  IS NULL OR overload_threshold  > 0);
ALTER TABLE settings ADD COLUMN underload_threshold REAL CHECK (underload_threshold IS NULL OR underload_threshold >= 0);
ALTER TABLE settings ADD COLUMN utilization_green   REAL CHECK (utilization_green   IS NULL OR (utilization_green   >= 0 AND utilization_green   <= 1.0));
ALTER TABLE settings ADD COLUMN utilization_yellow  REAL CHECK (utilization_yellow  IS NULL OR (utilization_yellow  >= 0 AND utilization_yellow  <= 1.0));
UPDATE settings SET overload_threshold = 1.10, underload_threshold = 0.50, utilization_green = 0.70, utilization_yellow = 1.00 WHERE id = 1;
