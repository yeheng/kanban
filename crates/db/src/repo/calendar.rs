use crate::error::DbError;
use crate::models::{Holiday, TimeOff, WeekTemplate};
use chrono::NaiveDate;
use sqlx::SqlitePool;

// ---- Work-week template ----
pub struct WeekTemplateRepo;
impl WeekTemplateRepo {
    /// Upsert the global template (design §3.3.9a; idx_wwt_global enforces one global row).
    ///
    /// `week[i]` is the per-day capacity fraction in `[0, 1.0]`; `0.0` marks a non-working
    /// day. The on/off bits are derived as `week[i] > 0`. The schema CHECK requires each
    /// `*_frac > 0`, so off-day fracs are stored as `1.0` (a sentinel that `frac_of` zeroes
    /// during hydration — the bit is the authoritative on/off signal).
    pub async fn upsert_global(
        pool: &SqlitePool,
        week: [f64; 7],
    ) -> Result<(), DbError> {
        crate::tx::with_write_tx(pool, |mut tx| Box::pin(async move {
            sqlx::query(
                "INSERT INTO work_week_template (scope, mon,tue,wed,thu,fri,sat,sun,
                   mon_frac,tue_frac,wed_frac,thu_frac,fri_frac,sat_frac,sun_frac)
                 VALUES ('global', ?,?,?,?,?,?,?,  ?,?,?,?,?,?,?)
                 ON CONFLICT DO UPDATE SET
                   mon=excluded.mon, tue=excluded.tue, wed=excluded.wed, thu=excluded.thu,
                   fri=excluded.fri, sat=excluded.sat, sun=excluded.sun,
                   mon_frac=excluded.mon_frac, tue_frac=excluded.tue_frac, wed_frac=excluded.wed_frac,
                   thu_frac=excluded.thu_frac, fri_frac=excluded.fri_frac, sat_frac=excluded.sat_frac,
                   sun_frac=excluded.sun_frac")
                .bind(week[0] > 0.0).bind(week[1] > 0.0).bind(week[2] > 0.0).bind(week[3] > 0.0)
                .bind(week[4] > 0.0).bind(week[5] > 0.0).bind(week[6] > 0.0)
                .bind(frac_col(week[0])).bind(frac_col(week[1])).bind(frac_col(week[2])).bind(frac_col(week[3]))
                .bind(frac_col(week[4])).bind(frac_col(week[5])).bind(frac_col(week[6]))
                .execute(&mut *tx).await?;
            Ok((tx, ()))
        })).await
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<WeekTemplate>, DbError> {
        Ok(sqlx::query_as::<_, WeekTemplate>(
            "SELECT id, scope, project_id, mon,tue,wed,thu,fri,sat,sun,
                    mon_frac,tue_frac,wed_frac,thu_frac,fri_frac,sat_frac,sun_frac \
             FROM work_week_template")
            .fetch_all(pool).await?)
    }
}

/// Fraction value to store for a day: the day's own fraction if it's a working day,
/// otherwise `1.0` (sentinel — `frac_of` zeroes it because the bit is 0). Keeps the
/// schema CHECK (`*_frac > 0`) satisfied for weekends.
fn frac_col(v: f64) -> f64 {
    if v > 0.0 { v } else { 1.0 }
}

// ---- Holidays ----
pub struct HolidayRepo;
impl HolidayRepo {
    pub async fn add(
        pool: &SqlitePool, project_id: Option<i64>, day: &str, fraction: f64, name: Option<&str>,
    ) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO holiday (project_id, day, fraction, name) VALUES (?,?,?,?) RETURNING id")
            .bind(project_id).bind(day).bind(fraction).bind(name).fetch_one(pool).await?;
        Ok(id)
    }
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Holiday>, DbError> {
        Ok(sqlx::query_as::<_, Holiday>(
            "SELECT id, project_id, day, fraction, name FROM holiday ORDER BY day")
            .fetch_all(pool).await?)
    }
}

// ---- Time off ----
pub struct TimeOffRepo;
impl TimeOffRepo {
    pub async fn add(
        pool: &SqlitePool, resource_id: i64, day: &str, fraction: f64, reason: Option<&str>,
    ) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO time_off (resource_id, day, fraction, reason) VALUES (?,?,?,?) RETURNING id")
            .bind(resource_id).bind(day).bind(fraction).bind(reason).fetch_one(pool).await?;
        Ok(id)
    }
    pub async fn list_for_resource(pool: &SqlitePool, resource_id: i64) -> Result<Vec<TimeOff>, DbError> {
        Ok(sqlx::query_as::<_, TimeOff>(
            "SELECT id, resource_id, day, fraction, reason FROM time_off WHERE resource_id = ? ORDER BY day")
            .bind(resource_id).fetch_all(pool).await?)
    }
}

// ---- Hydration: DB rows -> pure domain::Calendar ----

/// Effective per-day fraction: 0 when the day's bit is off, otherwise the stored fraction.
fn frac_of(bit: i64, f: f64) -> f64 { if bit == 0 { 0.0 } else { f } }

/// Load all calendar rows into a `domain::Calendar` (design §4.9 authoritative input).
/// This is the single bridge from persisted calendar state to the pure struct the
/// Phase 0 workload math runs on.
pub async fn hydrate(pool: &SqlitePool) -> Result<domain::Calendar, DbError> {
    let mut cal = domain::Calendar::default();
    for w in WeekTemplateRepo::list(pool).await? {
        let days = [
            frac_of(w.mon, w.mon_frac), frac_of(w.tue, w.tue_frac), frac_of(w.wed, w.wed_frac),
            frac_of(w.thu, w.thu_frac), frac_of(w.fri, w.fri_frac), frac_of(w.sat, w.sat_frac),
            frac_of(w.sun, w.sun_frac),
        ];
        let df = domain::DayFraction { days };
        match (w.scope.as_str(), w.project_id) {
            ("global", _) => cal.global_week = Some(df),
            ("project", Some(pid)) => { cal.project_weeks.insert(pid, df); }
            _ => {}
        }
    }
    for h in HolidayRepo::list(pool).await? {
        if let Ok(d) = NaiveDate::parse_from_str(&h.day, "%Y-%m-%d") {
            match h.project_id {
                Some(pid) => { cal.holidays_project.insert((pid, d), h.fraction); }
                None => { cal.holidays_global.insert(d, h.fraction); }
            }
        }
    }
    // time_off: hydrate all (small in MVP); a window-scoped query is a later optimization.
    let rows: Vec<(i64, String, f64)> = sqlx::query_as(
        "SELECT resource_id, day, fraction FROM time_off")
        .fetch_all(pool).await?;
    for (rid, day, frac) in rows {
        if let Ok(d) = NaiveDate::parse_from_str(&day, "%Y-%m-%d") {
            cal.time_off.insert((rid, d), frac);
        }
    }
    Ok(cal)
}
