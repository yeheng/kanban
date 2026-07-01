# HR Kanban — Phase 5: Reports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Assemble and export reports — resource utilization (R1), project budget burn (R3), AI decision records (R4, structured-only), and cost (R7) — to **CSV / Excel / PDF** at a user-chosen path, plus a JSON snapshot export.

**Architecture:** A `ReportService` builds a generic `ReportTable{columns, rows}` by reusing Phase 2/4 queries (workload, burn, AI runs, cost). Format generators (`csv`, `rust_xlsxwriter`, `printpdf`) turn a `ReportTable` into bytes. An `export_report` command writes bytes to a path the frontend picks via the Tauri save dialog. Decisions honored: reports are last (this phase), **CSV > Excel > PDF**, R4 stores structured fields only (confirmed #40), export to user-chosen path (#44), HTML→PDF optional (#41 — here we use `printpdf`, a pure-Rust lib, no Chromium).

**Tech Stack:** Rust (`csv`, `rust_xlsxwriter`, `printpdf`), Vue 3 + `@tauri-apps/plugin-dialog`, Vitest.

**Prerequisite:** Phase 2 (WorkloadService), Phase 4 (ai_optimization_runs) green.

**Scope note:** Reports were deliberately deferred to last (confirmed). PDF via `printpdf` (no `headless_chrome`/Chromium — keeps the binary lean; HTML→PDF remains a future optional feature #41). Snapshot >1MB externalization (#42) and Excel pivot/chart (#43) are noted impl-time refinements; here we export straightforward tables.

**Reference design:** `docs/design/2026-06-27-kanban-design.md` (§8 Reporting & Export).

---

## File Structure

```
kanban/
├── crates/app/Cargo.toml             # MOD: csv, rust_xlsxwriter, printpdf
├── crates/app/src/
│   ├── service/
│   │   ├── mod.rs                    # MOD: add reports
│   │   └── reports.rs                # NEW: ReportService + ReportTable + generators
│   ├── command.rs                    # MOD: export_report, export_snapshot
│   └── tests/reports.rs              # NEW
└── src/
    ├── api/index.ts                  # MOD: exportReport, exportSnapshot
    └── views/ReportsView.vue         # NEW (report center)
```

---

## Task 1: Report assembly (TDD)

**Files:**
- Create: `crates/app/src/service/reports.rs`
- Modify: `crates/app/src/service/mod.rs`
- Create: `crates/app/tests/reports.rs`

- [ ] **Step 1: `crates/app/src/service/reports.rs`**

```rust
use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ReportKind { ResourceUtilization, ProjectBurn, AiDecisions, Cost }

#[derive(Debug, Clone, Serialize)]
pub struct ReportTable { pub title: String, pub columns: Vec<String>, pub rows: Vec<Vec<String>> }

pub struct ReportService;
impl ReportService {
    pub async fn build(pool: &SqlitePool, kind: ReportKind, project_id: Option<i64>, start: &str, end: &str) -> Result<ReportTable, AppError> {
        match kind {
            ReportKind::ResourceUtilization => Self::resource_utilization(pool, start, end).await,
            ReportKind::ProjectBurn => Self::project_burn(pool).await,
            ReportKind::AiDecisions => Self::ai_decisions(pool).await,
            ReportKind::Cost => Self::cost(pool, project_id).await,
        }
    }

    async fn resource_utilization(pool: &SqlitePool, start: &str, end: &str) -> Result<ReportTable, AppError> {
        let mut rows = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let s = crate::service::workload::WorkloadService::resource_summary(pool, r.id, start, end).await?;
            rows.push(vec![r.name, fmt(s.capacity_pd), fmt(s.workload_pd), fmt(s.utilization), s.overloaded.to_string()]);
        }
        Ok(ReportTable { title: "Resource Utilization".into(),
            columns: vec!["resource","capacity_pd","workload_pd","utilization","overloaded"].into_iter().map(String::from).collect(),
            rows })
    }

    async fn project_burn(pool: &SqlitePool) -> Result<ReportTable, AppError> {
        let mut rows = Vec::new();
        for p in db::ProjectsRepo::list_active(pool).await? {
            let b = crate::service::workload::WorkloadService::project_burn(pool, p.id).await?;
            rows.push(vec![p.name, fmt(b.budget_pd), fmt(b.allocated_pd), fmt(b.usage)]);
        }
        Ok(ReportTable { title: "Project Budget Burn".into(),
            columns: vec!["project","budget_pd","allocated_pd","usage"].into_iter().map(String::from).collect(), rows })
    }

    /// R4: structured AI decision records only (no LLM prompt/response — confirmed #40).
    async fn ai_decisions(pool: &SqlitePool) -> Result<ReportTable, AppError> {
        let rows: Vec<(i64,String,i64,Option<f64>,String)> = sqlx::query_as(
            "SELECT id, status, applied, score_overall, created_at FROM ai_optimization_runs ORDER BY id DESC LIMIT 200")
            .fetch_all(pool).await?;
        Ok(ReportTable { title: "AI Decision Records".into(),
            columns: vec!["run_id","status","applied","score_overall","created_at"].into_iter().map(String::from).collect(),
            rows: rows.into_iter().map(|(id,st,ap,sc,ts)| vec![id.to_string(), st, ap.to_string(), sc.map(fmt).unwrap_or_default(), ts]).collect() })
    }

    /// R7: cost = Σ allocated_pd × effective_daily_rate(resource, project).
    async fn cost(pool: &SqlitePool, project_id: Option<i64>) -> Result<ReportTable, AppError> {
        let q = match project_id {
            Some(pid) => format!(
                "SELECT r.name, p.name, SUM(a.allocated_pd), \
                        COALESCE((SELECT rpr.daily_rate_pd FROM resource_project_rates rpr WHERE rpr.resource_id=r.id AND rpr.project_id=p.id ORDER BY rpr.valid_from DESC LIMIT 1), r.daily_rate_pd, 0) \
                 FROM allocations a JOIN resources r ON r.id=a.resource_id JOIN tasks t ON t.id=a.task_id JOIN projects p ON p.id=t.project_id \
                 WHERE p.id={} AND a.deleted_at IS NULL GROUP BY r.id, p.id", pid),
            None => "SELECT r.name, p.name, SUM(a.allocated_pd), \
                     COALESCE((SELECT rpr.daily_rate_pd FROM resource_project_rates rpr WHERE rpr.resource_id=r.id AND rpr.project_id=p.id ORDER BY rpr.valid_from DESC LIMIT 1), r.daily_rate_pd, 0) \
                     FROM allocations a JOIN resources r ON r.id=a.resource_id JOIN tasks t ON t.id=a.task_id JOIN projects p ON p.id=t.project_id \
                     WHERE a.deleted_at IS NULL GROUP BY r.id, p.id".to_string(),
        };
        let rows: Vec<(String,String,f64,Option<f64>)> = sqlx::query_as(&q).fetch_all(pool).await?;
        let mut out = Vec::new();
        let mut total = 0.0;
        for (res, proj, pd, rate) in rows {
            let cost = pd * rate.unwrap_or(0.0);
            total += cost;
            out.push(vec![res, proj, fmt(pd), rate.map(fmt).unwrap_or_else(|| "N/A".into()), fmt(cost)]);
        }
        out.push(vec!["TOTAL".into(), "".into(), "".into(), "".into(), fmt(total)]);
        Ok(ReportTable { title: "Cost".into(),
            columns: vec!["resource","project","allocated_pd","daily_rate_pd","cost"].into_iter().map(String::from).collect(), rows: out })
    }

    /// Workforce snapshot (JSON) — current utilization of all resources over a window.
    pub async fn snapshot_json(pool: &SqlitePool, start: &str, end: &str) -> Result<String, AppError> {
        let mut entries = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let s = crate::service::workload::WorkloadService::resource_summary(pool, r.id, start, end).await?;
            entries.push(serde_json::json!({ "resource": r.name, "capacity_pd": s.capacity_pd, "workload_pd": s.workload_pd, "utilization": s.utilization, "overloaded": s.overloaded }));
        }
        Ok(serde_json::to_string_pretty(&serde_json::json!({ "window": { "start": start, "end": end }, "resources": entries }))?)
    }
}

fn fmt(v: f64) -> String { format!("{:.2}", v) }
```

- [ ] **Step 2: Register — `crates/app/src/service/mod.rs`** add `pub mod reports;`

- [ ] **Step 3: Test — `crates/app/tests/reports.rs`**

```rust
use app::service::reports::{ReportKind, ReportService};
use app::service::projects::ProjectsService;
use db::pool::connect;
use db::AllocationsRepo;

#[tokio::test]
async fn resource_utilization_and_cost_reports() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 40.0).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name,daily_rate_pd) VALUES (1,'Alice',100.0)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,?,'T','2026-06-01','2026-07-31')").bind(pid).execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, "2026-06-29", "2026-07-03", 1.0).await.unwrap(); // 5 PD

    let ru = ReportService::build(&pool, ReportKind::ResourceUtilization, None, "2026-06-29", "2026-07-03").await.unwrap();
    assert!(ru.columns.contains(&"utilization".to_string()));
    assert_eq!(ru.rows.len(), 1);

    let cost = ReportService::build(&pool, ReportKind::Cost, Some(pid), "2026-06-29", "2026-07-03").await.unwrap();
    // last row is TOTAL; 5 PD * 100 = 500
    let total = cost.rows.last().unwrap().last().unwrap();
    assert!(total.contains("500"));
}

#[tokio::test]
async fn snapshot_json_is_valid() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let s = ReportService::snapshot_json(&pool, "2026-06-29", "2026-07-03").await.unwrap();
    assert!(s.contains("\"resources\""));
    serde_json::from_str::<serde_json::Value>(&s).unwrap();
}
```

- [ ] **Step 4: Run + commit**

```bash
cargo test -p app --test reports   # 2 passed
git add -A && git commit -m "feat(app): ReportService (utilization/burn/ai-decisions/cost + snapshot)"
```

---

## Task 2: CSV + Excel generators (TDD)

**Files:**
- Modify: `crates/app/Cargo.toml`, `crates/app/src/service/reports.rs`
- Modify: `crates/app/tests/reports.rs`

- [ ] **Step 1: `crates/app/Cargo.toml`** deps add:

```toml
csv = "1"
rust_xlsxwriter = "0.79"
```

- [ ] **Step 2: Generators — append to `crates/app/src/service/reports.rs`**

```rust
impl ReportService {
    pub fn to_csv(t: &ReportTable) -> Result<Vec<u8>, AppError> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(&t.columns).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
        for row in &t.rows { wtr.write_record(row).map_err(|e| crate::error::AppError::internal(e.to_string()))?; }
        Ok(wtr.into_inner().map_err(|e| crate::error::AppError::internal(e.to_string()))?)
    }

    pub fn to_xlsx(t: &ReportTable) -> Result<Vec<u8>, AppError> {
        use rust_xlsxwriter::*;
        let mut wb = Workbook::new();
        let sheet = wb.add_worksheet().set_name(&t.title).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
        for (c, col) in t.columns.iter().enumerate() {
            sheet.write_string(0, c as u16, col).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
        }
        for (r, row) in t.rows.iter().enumerate() {
            for (c, val) in row.iter().enumerate() {
                sheet.write_string((r + 1) as u32, c as u16, val).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
            }
        }
        Ok(wb.save_to_buffer().map_err(|e| crate::error::AppError::internal(e.to_string()))?)
    }
}
```

- [ ] **Step 3: Tests — append to `crates/app/tests/reports.rs`**

```rust
use app::service::reports::ReportTable;

#[test]
fn csv_has_header_and_rows() {
    let t = ReportTable { title: "X".into(), columns: vec!["a".into(), "b".into()], rows: vec![vec!["1".into(), "2".into()]] };
    let bytes = app::service::reports::ReportService::to_csv(&t).unwrap();
    let s = String::from_utf8(bytes).unwrap();
    assert!(s.contains("a,b"));
    assert!(s.contains("1,2"));
}

#[test]
fn xlsx_is_zip() {
    let t = ReportTable { title: "X".into(), columns: vec!["a".into()], rows: vec![vec!["1".into()]] };
    let bytes = app::service::reports::ReportService::to_xlsx(&t).unwrap();
    assert_eq!(&bytes[..2], b"PK"); // xlsx is a ZIP
}
```

- [ ] **Step 4: Run + commit**

```bash
cargo test -p app --test reports   # 4 passed
git add -A && git commit -m "feat(app): CSV + Excel report generators"
```

---

## Task 3: PDF generator + export command + snapshot export

**Files:**
- Modify: `crates/app/Cargo.toml`, `crates/app/src/service/reports.rs`, `crates/app/src/command.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: `crates/app/Cargo.toml`** — add PDF (optional, pure-Rust `printpdf`; no Chromium):

```toml
printpdf = { version = "0.7", optional = true, default-features = false, features = ["std"] }

[features]
default = []
pdf = ["dep:printpdf"]
```

- [ ] **Step 2: PDF generator — append to `crates/app/src/service/reports.rs`**

```rust
impl ReportService {
    #[cfg(feature = "pdf")]
    pub fn to_pdf(t: &ReportTable) -> Result<Vec<u8>, AppError> {
        use printpdf::*;
        let (doc, page1, mut layer1) = PdfDoc::new("report", &t.title, Mm(210.0), Mm(297.0));
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
        let mut y = 280.0;
        layer1.use_text(&t.title, 14.0, Mm(15.0), Mm(y), &font); y -= 10.0;
        layer1.use_text(&t.columns.join("   |   "), 9.0, Mm(15.0), Mm(y), &font); y -= 8.0;
        for row in &t.rows {
            if y < 20.0 { let (_, l) = doc.add_page(Mm(210.0), Mm(297.0)); layer1 = l; y = 280.0; }
            layer1.use_text(&row.join("   |   "), 9.0, Mm(15.0), Mm(y), &font); y -= 8.0;
        }
        Ok(doc.save_to_bytes().map_err(|e| crate::error::AppError::internal(e.to_string()))?)
    }
}
```

- [ ] **Step 3: Export commands — append to `crates/app/src/command.rs`**

```rust
use crate::service::reports::{ReportKind, ReportService};

#[derive(serde::Serialize)]
pub struct ExportResult { pub bytes_written: usize }

#[tauri::command]
pub async fn export_report(
    state: tauri::State<'_, AppState>, kind: ReportKind, project_id: Option<i64>,
    start: String, end: String, format: String, path: String,
) -> Result<ExportResult, AppError> {
    let table = ReportService::build(&state.pool, kind, project_id, &start, &end).await?;
    let bytes = match format.as_str() {
        "csv" => ReportService::to_csv(&table)?,
        "xlsx" => ReportService::to_xlsx(&table)?,
        #[cfg(feature = "pdf")]
        "pdf" => ReportService::to_pdf(&table)?,
        other => return Err(domain::DomainError::InvalidRatio(0.0).into()), // unsupported format
    };
    std::fs::write(&path, &bytes).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
    Ok(ExportResult { bytes_written: bytes.len() })
}

#[tauri::command]
pub async fn export_snapshot(state: tauri::State<'_, AppState>, start: String, end: String, path: String) -> Result<(), AppError> {
    let json = ReportService::snapshot_json(&state.pool, &start, &end).await?;
    std::fs::write(&path, json).map_err(|e| crate::error::AppError::internal(e.to_string()))?;
    Ok(())
}
```

- [ ] **Step 4: Register in `src-tauri/src/main.rs`** — add `export_report, export_snapshot` to the handler list. (Build with `-p kanban --features app/pdf` when PDF is wanted.)

- [ ] **Step 5: Build (default, no PDF) + full suite**

Run: `cargo build --workspace && cargo test --workspace`
Expected: green (reports 4 passed; PDF path compiled only with `--features app/pdf`).

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat(app): PDF generator (optional) + export_report/export_snapshot commands"
```

---

## Task 4: Report center frontend

**Files:**
- Modify: `package.json` (add `@tauri-apps/plugin-dialog`)
- Modify: `src-tauri/tauri.conf.json` + capabilities (dialog plugin)
- Modify: `src/api/index.ts`
- Create: `src/views/ReportsView.vue`
- Modify: `src/router.ts`, nav

- [ ] **Step 1: Dialog plugin** — `package.json` deps add `"@tauri-apps/plugin-dialog": "^2.0.0"`; `npm install`. In `src-tauri/Cargo.toml` add `tauri-plugin-dialog = "2"` and in `main.rs` `.plugin(tauri_plugin_dialog::init())`. Add to `tauri.conf.json` `"app.permissions"` or a capabilities file: `"dialog:allow-save"`.

- [ ] **Step 2: API — append to `src/api/index.ts`**

```ts
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs"; // only if exporting from JS; we use Rust path here

export const reportKinds = ["ResourceUtilization","ProjectBurn","AiDecisions","Cost"] as const;
export type ReportKind = typeof reportKinds[number];

export const reports = {
  async exportReport(kind: ReportKind, projectId: number | null, start: string, end: string, format: "csv" | "xlsx" | "pdf") {
    const ext = format;
    const path = await save({ defaultPath: `${kind}.${ext}`, filters: [{ name: format, extensions: [ext] }] });
    if (!path) return false;
    await invoke("export_report", { kind, projectId, start, end, format, path });
    return true;
  },
  async exportSnapshot(start: string, end: string) {
    const path = await save({ defaultPath: "workforce-snapshot.json", filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!path) return false;
    await invoke("export_snapshot", { start, end, path });
    return true;
  },
};
```

> `@tauri-apps/plugin-fs` isn't required since the Rust command writes the file; `save()` returns a path string only. Remove the unused `writeTextFile` import.

- [ ] **Step 3: `src/views/ReportsView.vue`**

```vue
<script setup lang="ts">
import { ref } from "vue";
import { reports, reportKinds, type ReportKind } from "../api";
const kind = ref<ReportKind>("ResourceUtilization");
const start = ref("2026-06-29"); const end = ref("2026-07-12");
const fmt = ref<"csv" | "xlsx" | "pdf">("csv");
const msg = ref("");
const cn: Record<ReportKind,string> = { ResourceUtilization: "资源利用率", ProjectBurn: "项目预算消耗", AiDecisions: "AI 决策记录", Cost: "成本" };
async function doExport() {
  const ok = await reports.exportReport(kind.value, null, start.value, end.value, fmt.value);
  msg.value = ok ? `已导出 ${kind.value}.${fmt.value}` : "已取消";
}
async function doSnapshot() {
  const ok = await reports.exportSnapshot(start.value, end.value);
  msg.value = ok ? "已导出快照 JSON" : "已取消";
}
</script>
<template>
  <h2 style="margin-top:0">报表 / Reports</h2>
  <select v-model="kind"><option v-for="k in reportKinds" :key="k" :value="k">{{ cn[k] }}</option></select>
  窗口 <input v-model="start" type="date" /> – <input v-model="end" type="date" />
  格式 <select v-model="fmt"><option value="csv">CSV</option><option value="xlsx">Excel</option><option value="pdf">PDF</option></select>
  <button @click="doExport">导出报表</button>
  <button @click="doSnapshot">导出人力快照 (JSON)</button>
  <p>{{ msg }}</p>
</template>
```

- [ ] **Step 4: Route + nav** — add `{ path: "/reports", component: () => import("./views/ReportsView.vue") }` + nav link.

- [ ] **Step 5: Build + E2E + commit**

```bash
npm run build
# run app; export a CSV (resource utilization) → file written to chosen path; open it.
git add -A && git commit -m "feat(web): report center (export CSV/Excel/PDF + snapshot)"
```

---

## Self-Review

**Spec coverage (§8 + roadmap Phase 5):**
- R1 resource utilization, R3 project burn, R4 AI decision records, R7 cost → Task 1 ✓
- Export formats CSV/Excel/PDF, priority CSV>Excel>PDF → Tasks 2–3 ✓
- R4 structured-only (no prompt/response) — confirmed #40 → Task 1 ✓
- Export to user-chosen path (save dialog) — confirmed #44 → Task 4 ✓
- PDF via pure-Rust `printpdf` (no Chromium; HTML→PDF remains optional #41) → Task 3 ✓
- Workforce snapshot (JSON) → Tasks 1, 4 ✓

**Deferred (not placeholders):** Excel pivot/chart (#43) and snapshot >1MB externalization (#42) — impl-time refinements on the straightforward tables produced here; multi-page PDF pagination is minimal.

**Placeholder scan:** none — complete code; tests assert CSV content, xlsx ZIP magic, cost total (5×100=500), snapshot JSON validity.

**Type consistency:** `ReportKind` enum maps 1:1 TS↔Rust; `ReportTable{columns,rows}` consumed identically by all generators; `export_report(kind, projectId, start, end, format, path)` camelCase→snake_case params.

**Known impl-time items:** PDF feature must be enabled (`--features app/pdf`) and `printpdf` API verified for the locked version; the cost query inlines `effective_daily_rate` via a correlated subquery (resource_project_rates → resources.daily_rate_pd → 0) — matches design §8 but could be factored into a repo method.

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-06-27-kanban-phase5-reports.md`. **1. Subagent-Driven** (recommended) or **2. Inline**. Next: **Phase 6 (polish)** — the final plan.
