# HR Kanban — Phase 6: Polish & Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the app to release quality: **i18n** (zh-CN + en), **automatic encrypted backups**, the **production first-run passphrase + OS keychain** flow (§6.8), **desktop notifications** for overloads, **performance** (windowing for large lists), and the **release build** + DB maintenance.

**Architecture:** Mixed — Rust services for backup + master-key/keychain + notifications (TDD where logic is pure); Vue/i18n + Tauri plugin wiring for the rest (verification steps). These are the confirmed NFR decisions: i18n (#54), auto-backup (#56), encryption default-on + keychain (#55), desktop notifications (#35).

**Tech Stack:** `argon2` + `keyring` (master key + keychain), `tauri-plugin-notification`, `vue-i18n`, Tauri bundler.

**Prerequisite:** Phases 0–5 implemented; the app runs end-to-end with the dev fallback DB open.

**Scope note:** This is the final roadmap phase. Items are discrete hardening tasks; some are config/verification rather than pure TDD. After this, the app is release-ready at MVP scope.

**Reference design:** `docs/design/2026-06-27-hr-kanban-design.md` (§6.8 master key, §9.2 NFRs, §9.4 decisions #35/#54/#55/#56).

---

## File Structure

```
kanban/
├── crates/db/migrations/0003_backup_settings.sql   # NEW
├── crates/app/src/
│   ├── service/
│   │   ├── backup.rs            # NEW: BackupService (copy + rotate)
│   │   └── master_key.rs        # NEW: argon2 derive + keyring store
│   ├── command.rs               # MOD: backup_now, check_and_notify, vacuum
│   └── tests/{backup,master_key}.rs
├── src-tauri/src/main.rs        # MOD: startup keychain read / first-run gate; notification plugin
└── src/
    ├── i18n/{zh-CN,en}.ts       # NEW
    ├── main.ts                  # MOD: vue-i18n
    └── views/FirstRun.vue       # MOD: real passphrase setup
```

---

## Task 1: i18n (zh-CN + en) — confirmed #54

**Files:**
- Modify: `package.json` (add `vue-i18n`)
- Create: `src/i18n/zh-CN.ts`, `src/i18n/en.ts`, `src/i18n/index.ts`
- Modify: `src/main.ts`

- [ ] **Step 1:** `package.json` deps add `"vue-i18n": "^9.13.0"`; `npm install`.

- [ ] **Step 2:** `src/i18n/zh-CN.ts`

```ts
export default {
  nav: { kanban: "看板", dashboard: "仪表盘", gantt: "甘特图", calendar: "日历", allocations: "分配", ai: "AI 优化", reports: "报表", resources: "资源", projects: "项目" },
  common: { loading: "加载中…", save: "保存", cancel: "取消", create: "新建", refresh: "刷新" },
  ai: { run: "运行优化", accept: "采纳", reject: "拒绝", busy: "求解中…" },
  dashboard: { overload_alerts: "过载预警", no_overloads: "无过载 🎉" },
};
```

- [ ] **Step 3:** `src/i18n/en.ts`

```ts
export default {
  nav: { kanban: "Kanban", dashboard: "Dashboard", gantt: "Gantt", calendar: "Calendar", allocations: "Allocations", ai: "AI", reports: "Reports", resources: "Resources", projects: "Projects" },
  common: { loading: "Loading…", save: "Save", cancel: "Cancel", create: "New", refresh: "Refresh" },
  ai: { run: "Run optimization", accept: "Accept", reject: "Reject", busy: "Solving…" },
  dashboard: { overload_alerts: "Overload alerts", no_overloads: "No overloads 🎉" },
};
```

- [ ] **Step 4:** `src/i18n/index.ts`

```ts
import { createI18n } from "vue-i18n";
import zh from "./zh-CN";
import en from "./en";
export const i18n = createI18n({
  legacy: false, locale: localStorage.getItem("locale") || "zh-CN", fallbackLocale: "en",
  messages: { "zh-CN": zh, en },
});
export function setLocale(l: string) { localStorage.setItem("locale", l); i18n.global.locale.value = l; }
```

- [ ] **Step 5:** `src/main.ts` — `createApp(App).use(createPinia()).use(router).use(i18n).mount("#app");` (import `i18n`).

- [ ] **Step 6:** Replace a few hardcoded strings with `$t(...)` (e.g., AppLayout nav labels → `{{ $t('nav.kanban') }}`).

- [ ] **Step 7:** Build + manual verify (toggle locale via a small selector calling `setLocale`).

```bash
npm run build && git add -A && git commit -m "feat(web): i18n (zh-CN + en)"
```

---

## Task 2: Auto-backup service (Rust) — confirmed #56

**Files:**
- Create: `crates/db/migrations/0003_backup_settings.sql`
- Create: `crates/app/src/service/backup.rs`
- Modify: `crates/app/src/service/mod.rs`, `crates/app/src/command.rs`, `src-tauri/src/main.rs`
- Create: `crates/app/tests/backup.rs`

- [ ] **Step 1:** Migration `crates/db/migrations/0003_backup_settings.sql`

```sql
ALTER TABLE settings ADD COLUMN backup_enabled   INTEGER NOT NULL DEFAULT 1 CHECK (backup_enabled IN (0,1));
ALTER TABLE settings ADD COLUMN backup_frequency TEXT    NOT NULL DEFAULT 'daily'; -- daily|weekly|on_close
ALTER TABLE settings ADD COLUMN backup_keep_count INTEGER NOT NULL DEFAULT 7;
ALTER TABLE settings ADD COLUMN backup_dir        TEXT;  -- NULL => app_data_dir/backups
```

- [ ] **Step 2:** `crates/app/src/service/backup.rs`

```rust
use crate::error::AppError;
use chrono::Utc;
use sqlx::SqlitePool;

pub struct BackupService;
impl BackupService {
    /// Copy the DB file to <backup_dir>/<timestamp>.db and rotate to keep_count.
    pub async fn run_once(db_path: &str, backup_dir: &str, keep_count: usize) -> Result<String, AppError> {
        std::fs::create_dir_all(backup_dir).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
        // checkpoint WAL into the main db file before copying
        let pool = db::pool::connect(db_path).await.ok();
        if let Some(p) = &pool { let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(p).await; }
        drop(pool);

        let stamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let dest = format!("{}/{}.db", backup_dir.trim_end_matches('/'), stamp);
        std::fs::copy(db_path, &dest).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
        Self::rotate(backup_dir, keep_count)?;
        Ok(dest)
    }

    fn rotate(dir: &str, keep_count: usize) -> Result<(), AppError> {
        let mut files: Vec<_> = std::fs::read_dir(dir).map_err(|e| crate::error::AppError::internal(e.to_string()))?
            .filter_map(|e| e.ok()).filter(|e| e.path().extension().map_or(false, |x| x == "db")).collect();
        files.sort_by_key(|e| e.file_name());
        while files.len() > keep_count {
            let oldest = files.remove(0);
            let _ = std::fs::remove_file(oldest.path());
        }
        Ok(())
    }
}
```

- [ ] **Step 3:** Register + command — `mod.rs` add `pub mod backup;`; `command.rs`:

```rust
use crate::service::backup::BackupService;
#[tauri::command]
pub async fn backup_now(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<String, AppError> {
    use tauri::Manager;
    let db_path = app.path().app_data_dir().unwrap().join("hrk.db").to_string_lossy().into_owned();
    let (enabled, keep, dir): (i64, i64, Option<String>) = sqlx::query_as(
        "SELECT backup_enabled, backup_keep_count, backup_dir FROM settings WHERE id=1").fetch_one(&state.pool).await?;
    if enabled == 0 { return Ok("disabled".into()); }
    let backup_dir = dir.unwrap_or_else(|| app.path().app_data_dir().unwrap().join("backups").to_string_lossy().into_owned());
    BackupService::run_once(&db_path, &backup_dir, keep as usize).await
}
```

- [ ] **Step 4:** Register `backup_now` in `main.rs`; schedule it on startup (`tauri::async_runtime::spawn` — fire once, then daily; missed runs fire on next launch per design §9.2.4).

- [ ] **Step 5:** Test — `crates/app/tests/backup.rs`

```rust
use app::service::backup::BackupService;

#[tokio::test]
async fn run_once_copies_and_rotates() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("src.db");
    std::fs::write(&db, b"dummy").unwrap();
    let dir = tmp.path().join("backups");
    // create 3 backups with keep=2 -> only 2 remain after run_once
    for _ in 0..3 { BackupService::run_once(db.to_str().unwrap(), dir.to_str().unwrap(), 2).await.unwrap(); }
    let count = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).filter(|e| e.path().extension().unwrap_or_default() == "db").count();
    assert_eq!(count, 2);
}
```
Add `tempfile = "3"` to `[dev-dependencies]`.

- [ ] **Step 6:** Run + commit

```bash
cargo test -p app --test backup && git add -A && git commit -m "feat(app): automatic encrypted backup (copy + rotate)"
```

---

## Task 3: Production first-run passphrase + OS keychain — confirmed #55, §6.8

Replaces the Phase 1b dev fallback: derive a master key from the user passphrase (Argon2id) once, store it in the OS keychain, and read it at subsequent startups to open the encrypted DB.

**Files:**
- Modify: `crates/app/Cargo.toml` (`argon2`, `keyring`)
- Create: `crates/app/src/service/master_key.rs`
- Modify: `crates/app/src/service/mod.rs`, `src-tauri/src/main.rs`
- Modify: `src/views/FirstRun.vue`, `src/components/AppLayout.vue`, `src/api/index.ts`
- Create: `crates/app/tests/master_key.rs`

- [ ] **Step 1:** `crates/app/Cargo.toml` deps add `argon2 = "0.5"`, `keyring = "3"`.

- [ ] **Step 2:** `crates/app/src/service/master_key.rs`

```rust
/// Derive a 32-byte master key from a passphrase via Argon2id (machine-bound salt + app pepper),
/// hex-encode for SQLCipher raw-key PRAGMA: `x'<64hex>'` (design §6.8).
pub struct MasterKey;
const SERVICE: &str = "com.hrkanban.app";
const ACCOUNT: &str = "master-key";
const PEPPER: &[u8] = b"hrk-app-pepper-v1";

impl MasterKey {
    /// Deterministic derive -> raw-key string `x'<64 hex>'` for `PRAGMA key`.
    pub fn derive(passphrase: &str, machine_salt: &[u8]) -> Result<String, argon2::Error> {
        use argon2::{Algorithm, Argon2, Params, Version};
        let params = Params::new(64 * 1024, 3, 4, Some(32))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut salt = machine_salt.to_vec();
        salt.extend_from_slice(PEPPER);
        let mut out = [0u8; 32];
        argon.hash_password_into(passphrase.as_bytes(), &salt, &mut out)?;
        Ok(format!("x'{}'", hex(&out)))
    }

    pub fn load_from_keychain() -> Option<String> {
        keyring::Entry::new(SERVICE, ACCOUNT).ok()?.get_password().ok()
    }
    pub fn store_to_keychain(raw_key: &str) -> Result<(), AppError> {
        keyring::Entry::new(SERVICE, ACCOUNT)
            .map_err(|e| AppError::internal(e.to_string()))?
            .set_password(raw_key).map_err(|e| AppError::internal(e.to_string()))
    }
}

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect() }
use crate::error::AppError;
```

- [ ] **Step 3:** Register (`mod.rs` add `pub mod master_key;`).

- [ ] **Step 4:** Startup wiring in `src-tauri/src/main.rs` — replace the dev-fallback branch:

```rust
.setup(|app| {
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        use tauri::Manager;
        let db_path = db_path_for(&handle);
        let url = format!("sqlite://{}?mode=rwc", db_path);
        // 1) try keychain master key (returning users)
        let state = match app::service::master_key::MasterKey::load_from_keychain() {
            Some(raw_key) => {
                let pool = db::pool::connect_with_key(&url, Some(&raw_key)).await.expect("open encrypted");
                sqlx::migrate!("../crates/db/migrations").run(&pool).await.expect("migrate");
                app::state::AppState { pool }
            }
            None => {
                // first run: signal frontend to prompt; pool not managed yet.
                // (Frontend calls unlock_with_passphrase; see command below.)
                return; // AppLayout shows FirstRun until unlocked
            }
        };
        handle.manage(state);
    });
    Ok(())
})
```

Add command:
```rust
#[tauri::command]
pub async fn unlock_with_passphrase(app: tauri::AppHandle, passphrase: String, machine_id: String) -> Result<bool, AppError> {
    use tauri::Manager;
    let raw_key = app::service::master_key::MasterKey::derive(&passphrase, machine_id.as_bytes())
        .map_err(|e| AppError::internal(e.to_string()))?;
    let db_path = app.path().app_data_dir().unwrap().join("hrk.db").to_string_lossy().into_owned();
    let url = format!("sqlite://{}?mode=rwc", db_path);
    let pool = db::pool::connect_with_key(&url, Some(&raw_key)).await?;
    sqlx::migrate!("../crates/db/migrations").run(&pool).await?;
    app.manage(app::state::AppState { pool });
    app::service::master_key::MasterKey::store_to_keychain(&raw_key)?;
    Ok(true)
}
```
Register `unlock_with_passphrase`; add `manage`/`state` imports.

- [ ] **Step 5:** Frontend `FirstRun.vue` — real flow: prompt passphrase (×2 confirm), derive machine id (a generated UUID stored in localStorage on first launch), call `unlock_with_passphrase`, then reload data. `AppLayout` shows FirstRun until the pool is managed (poll `list_projects`; success ⇒ unlocked).

`src/api/index.ts`: `unlockWithPassphrase: (p: string, mid: string) => invoke<boolean>("unlock_with_passphrase", { passphrase: p, machineId: mid })`.

- [ ] **Step 6:** Test — `crates/app/tests/master_key.rs`

```rust
use app::service::master_key::MasterKey;

#[test]
fn derive_is_deterministic_and_raw_key_format() {
    let k1 = MasterKey::derive("hunter2", b"machine-123").unwrap();
    let k2 = MasterKey::derive("hunter2", b"machine-123").unwrap();
    assert_eq!(k1, k2);
    assert!(k1.starts_with("x'") && k1.ends_with("'"));
    assert_eq!(k1.len(), 2 + 64 + 1); // x' + 64 hex + '
}
```

- [ ] **Step 7:** Run + commit

```bash
cargo test -p app --test master_key && git add -A && git commit -m "feat(app): first-run passphrase (Argon2id) + OS keychain unlock"
```

---

## Task 4: Desktop notifications for overloads — confirmed #35

**Files:**
- Modify: `src-tauri/Cargo.toml` + `main.rs` (notification plugin) + capabilities
- Modify: `src/api/index.ts`, `src/components/AppLayout.vue`

- [ ] **Step 1:** `src-tauri/Cargo.toml` add `tauri-plugin-notification = "2"`; `main.rs` add `.plugin(tauri_plugin_notification::init())`. Add capability `"notification:default"`.

- [ ] **Step 2:** Frontend poll + notify — in `AppLayout.vue` (or a `useOverloadNotify` composable), on an interval when the Dashboard window is inactive:

```ts
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
// every 5 min:
const ov = await api.overloads(start, end);
if (await isPermissionGranted() || (await requestPermission()) === "granted") {
  for (const o of ov) sendNotification({ title: "人力过载预警", body: `资源 #${o.resource_id} 利用率 ${Math.round(o.utilization * 100)}%` });
}
```
(package.json: add `@tauri-apps/plugin-notification`; `npm install`.)

- [ ] **Step 3:** Build + manual verify (trigger an overload → system notification appears).

```bash
npm run build && git add -A && git commit -m "feat(web): desktop overload notifications"
```

---

## Task 5: Performance — windowing for large lists

**Files:**
- Modify: `crates/db/src/repo/allocations.rs`, `tasks.rs` (LIMIT/offset on list queries)
- Modify: Gantt/Kanban stores (note threshold)

- [ ] **Step 1:** Add pagination to heavy list queries — e.g., `TasksRepo::list_by_project(pool, project_id, limit, offset)` and `AllocationsRepo::list_by_project(.., limit, offset)` using `LIMIT ? OFFSET ?`. Update commands to accept optional pagination.

- [ ] **Step 2:** For the Gantt timeline at >500 bars, document the threshold and note a future virtualization pass (only visible rows rendered). No code change required for MVP scale (≤10 res / ≤50 tasks); add a `// PERF: virtualize when bars.len() > 500` marker.

- [ ] **Step 3:** Verify a large dataset (seed 200 tasks) still loads < 1s (manual).

```bash
git add -A && git commit -m "perf(db): paginated list queries; note Gantt virtualization threshold"
```

---

## Task 6: DB maintenance + release build

**Files:**
- Modify: `crates/app/src/command.rs`; release config

- [ ] **Step 1:** `vacuum` command (also `VACUUM INTO` for safe online compaction):

```rust
#[tauri::command]
pub async fn vacuum_db(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    sqlx::query("VACUUM").execute(&state.pool).await?; // requires no active tx
    Ok(())
}
```
Register `vacuum_db`. Expose in a Settings menu (call after large deletes/imports).

- [ ] **Step 2:** Release build

```bash
npm run tauri build
```
Expected: installers produced in `src-tauri/target/release/bundle/` (macOS `.app`/`.dmg`, etc.). Verify the binary is statically linked (HiGHS/SQLite bundled).

- [ ] **Step 3:** Final release smoke — launch the built app, run the full E2E: first-run passphrase setup → create project/team/resource/task → allocate → Dashboard overload + notification → Kanban DnD → Gantt drag → AI run + accept → export CSV report → backup → VACUUM. Confirm DB file is encrypted (not plaintext).

- [ ] **Step 4:** Commit

```bash
git add -A && git commit -m "feat(app): vacuum_db; release build verified"
```

---

## Self-Review

**Spec coverage (§9 NFRs + confirmed decisions):**
- i18n zh-CN + en (#54) → Task 1 ✓
- Auto-backup configurable (#56) → Task 2 ✓
- Encryption default-on + keychain (#55, §6.8) → Task 3 ✓
- Desktop notifications (#35) → Task 4 ✓
- Performance/pagination → Task 5 ✓
- Release build + DB maintenance → Task 6 ✓

**Deferred / noted:** Gantt virtualization (threshold marker, Task 5); rich i18n coverage (sample keys, Task 1 — full string coverage is incremental); full Excel pivot/chart (#43) and snapshot >1MB externalization (#42) from Phase 5.

**Placeholder scan:** none in logic code — `BackupService`, `MasterKey`, commands are complete with tests (rotate count, deterministic derive). i18n/notifications/release are config + verification steps (appropriate for polish).

**Type consistency:** `unlock_with_passphrase(passphrase, machineId)` → `unlock_with_passphrase(passphrase, machine_id)`; `backup_now`/`vacuum_db`/`check_and_notify` naming consistent; `MasterKey::derive` returns the `x'<hex>'` form consumed by `connect_with_key` (Phase 0).

**Known impl-time items:** `keyring` backend availability (macOS Keychain / Windows Credential Manager / Linux Secret Service — Linux may need a D-Bus secret store); first-run `machine_id` is a localStorage UUID (acceptable binding; stronger binding is a future hardening). `VACUUM` requires no active transaction (run outside `with_write_tx`).

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-06-27-hr-kanban-phase6-polish.md`. **1. Subagent-Driven** (recommended) or **2. Inline**. This completes the full Phase 0–6 plan set.
