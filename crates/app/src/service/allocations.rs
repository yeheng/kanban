use crate::error::AppError;
use crate::service::dates;
use chrono::{Days, NaiveDate};
use db::models::AllocationView;
use db::AllocationsRepo;
use std::collections::HashMap;
use sqlx::SqlitePool;

pub struct AllocationsService;

impl AllocationsService {
    pub async fn create(
        pool: &SqlitePool,
        resource_id: i64,
        task_id: i64,
        start: &str,
        end: &str,
        percent: f64,
    ) -> Result<i64, AppError> {
        validate_percent(percent)?;
        let window = dates::required_window(start, end)?;
        validate_allocation_constraints(
            pool,
            None,
            resource_id,
            task_id,
            &window.start,
            &window.end,
            percent,
        )
        .await?;
        Ok(AllocationsRepo::create(
            pool,
            resource_id,
            task_id,
            &window.start,
            &window.end,
            percent,
        )
        .await?)
    }

    pub async fn list_by_project(
        pool: &SqlitePool,
        project_id: i64,
    ) -> Result<Vec<AllocationView>, AppError> {
        Ok(AllocationsRepo::list_by_project(pool, project_id).await?)
    }

    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        start: &str,
        end: &str,
        percent: f64,
    ) -> Result<(), AppError> {
        validate_percent(percent)?;
        let window = dates::required_window(start, end)?;
        let (resource_id, task_id): (i64, i64) = sqlx::query_as(
            "SELECT resource_id, task_id FROM allocations WHERE id=? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| domain::DomainError::NotFound(format!("allocation {}", id)))?;
        validate_allocation_constraints(
            pool,
            Some(id),
            resource_id,
            task_id,
            &window.start,
            &window.end,
            percent,
        )
        .await?;
        Ok(AllocationsRepo::update(pool, id, &window.start, &window.end, percent).await?)
    }

    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
        Ok(AllocationsRepo::soft_delete(pool, id).await?)
    }
}

fn validate_percent(percent: f64) -> Result<(), AppError> {
    if percent.is_finite() && percent > 0.0 && percent <= 1.0 {
        Ok(())
    } else {
        Err(domain::DomainError::InvalidRatio(percent).into())
    }
}

async fn validate_allocation_constraints(
    pool: &SqlitePool,
    self_id: Option<i64>,
    resource_id: i64,
    task_id: i64,
    start: &str,
    end: &str,
    percent: f64,
) -> Result<(), AppError> {
    validate_capacity(pool, self_id, resource_id, task_id, start, end, percent).await?;
    validate_dependencies(pool, task_id, start, end).await?;
    Ok(())
}

async fn validate_capacity(
    pool: &SqlitePool,
    self_id: Option<i64>,
    resource_id: i64,
    task_id: i64,
    start: &str,
    end: &str,
    percent: f64,
) -> Result<(), AppError> {
    let start_date = parse_date(start)?;
    let end_date = parse_date(end)?;
    let (project_id,): (i64,) = sqlx::query_as(
        "SELECT project_id FROM tasks WHERE id=? AND deleted_at IS NULL",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| domain::DomainError::NotFound(format!("task {}", task_id)))?;
    let cal = db::repo::calendar::hydrate(pool).await?;

    let rows: Vec<(i64, NaiveDate, NaiveDate, f64)> = sqlx::query_as(
        "SELECT id, start_date, end_date, percent \
         FROM allocations \
         WHERE resource_id=? AND deleted_at IS NULL AND status <> 'cancelled' \
           AND (? IS NULL OR id <> ?) \
           AND start_date <= ? AND end_date >= ?",
    )
    .bind(resource_id)
    .bind(self_id)
    .bind(self_id)
    .bind(end)
    .bind(start)
    .fetch_all(pool)
    .await?;

    let mut load_by_day: HashMap<NaiveDate, f64> = HashMap::new();
    for (_id, s, e, existing_percent) in rows {
        let mut d = s.max(start_date);
        let last = e.min(end_date);
        while d <= last {
            if cal.day_factor(project_id, resource_id, d) > 0.0 {
                *load_by_day.entry(d).or_default() += existing_percent;
            }
            d = d.checked_add_days(Days::new(1)).unwrap();
        }
    }

    let mut d = start_date;
    while d <= end_date {
        let limit = cal.day_factor(project_id, resource_id, d);
        if limit > 0.0 {
            let load = load_by_day.get(&d).copied().unwrap_or(0.0) + percent;
            if load > limit + 1e-9 {
                return Err(domain::DomainError::InsufficientCapacity {
                    resource_id,
                    shortfall_pd: load - limit,
                }
                .into());
            }
        }
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
    Ok(())
}

async fn validate_dependencies(
    pool: &SqlitePool,
    task_id: i64,
    start: &str,
    end: &str,
) -> Result<(), AppError> {
    let start_date = parse_date(start)?;
    let end_date = parse_date(end)?;

    let predecessors: Vec<(i64, i64, Option<NaiveDate>)> = sqlx::query_as(
        "SELECT d.predecessor_id, d.lag_days, COALESCE(MAX(a.end_date), tp.end_date) \
         FROM task_dependencies d \
         JOIN tasks tp ON tp.id = d.predecessor_id AND tp.deleted_at IS NULL \
         LEFT JOIN allocations a ON a.task_id = d.predecessor_id \
             AND a.deleted_at IS NULL AND a.status <> 'cancelled' \
         WHERE d.task_id = ? \
         GROUP BY d.predecessor_id, d.lag_days, tp.end_date",
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;
    for (predecessor_id, lag_days, predecessor_end) in predecessors {
        if let Some(predecessor_end) = predecessor_end {
            let earliest = add_days(predecessor_end, lag_days)?;
            if start_date < earliest {
                return Err(domain::DomainError::DependencyViolation {
                    task_id,
                    related_task_id: predecessor_id,
                }
                .into());
            }
        }
    }

    let successors: Vec<(i64, i64, Option<NaiveDate>)> = sqlx::query_as(
        "SELECT d.task_id, d.lag_days, COALESCE(MIN(a.start_date), ts.start_date) \
         FROM task_dependencies d \
         JOIN tasks ts ON ts.id = d.task_id AND ts.deleted_at IS NULL \
         LEFT JOIN allocations a ON a.task_id = d.task_id \
             AND a.deleted_at IS NULL AND a.status <> 'cancelled' \
         WHERE d.predecessor_id = ? \
         GROUP BY d.task_id, d.lag_days, ts.start_date",
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;
    for (successor_id, lag_days, successor_start) in successors {
        if let Some(successor_start) = successor_start {
            let earliest_successor = add_days(end_date, lag_days)?;
            if successor_start < earliest_successor {
                return Err(domain::DomainError::DependencyViolation {
                    task_id,
                    related_task_id: successor_id,
                }
                .into());
            }
        }
    }

    Ok(())
}

fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| domain::DomainError::InvalidDateWindow.into())
}

fn add_days(date: NaiveDate, lag_days: i64) -> Result<NaiveDate, AppError> {
    if lag_days >= 0 {
        date.checked_add_days(Days::new(lag_days as u64))
    } else {
        date.checked_sub_days(Days::new((-lag_days) as u64))
    }
    .ok_or_else(|| domain::DomainError::InvalidDateWindow.into())
}
