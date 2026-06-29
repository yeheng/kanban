use crate::error::DbError;
use crate::models::{Team, TeamMember, TeamOverride};
use sqlx::SqlitePool;

pub struct TeamsRepo;

impl TeamsRepo {
    pub async fn create(
        pool: &SqlitePool,
        name: &str,
        description: Option<&str>,
    ) -> Result<i64, DbError> {
        let (id,): (i64,) =
            sqlx::query_as("INSERT INTO teams (name, description) VALUES (?,?) RETURNING id")
                .bind(name)
                .bind(description)
                .fetch_one(pool)
                .await?;
        Ok(id)
    }
    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Team, DbError> {
        sqlx::query_as::<_, Team>(
            "SELECT id, name, description FROM teams WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(DbError::NotFound)
    }
    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Team>, DbError> {
        Ok(sqlx::query_as::<_, Team>(
            "SELECT id, name, description FROM teams WHERE deleted_at IS NULL ORDER BY name",
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE teams SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') \
             WHERE id = ? AND deleted_at IS NULL")
            .bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }
}

pub struct TeamMembersRepo;
impl TeamMembersRepo {
    pub async fn add(
        pool: &SqlitePool,
        team_id: i64,
        resource_id: i64,
        role: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO team_members (team_id, resource_id, role) VALUES (?,?,?) \
             ON CONFLICT(team_id, resource_id) DO UPDATE SET role = excluded.role",
        )
        .bind(team_id)
        .bind(resource_id)
        .bind(role)
        .execute(pool)
        .await?;
        Ok(())
    }
    pub async fn list_members(pool: &SqlitePool, team_id: i64) -> Result<Vec<TeamMember>, DbError> {
        Ok(sqlx::query_as::<_, TeamMember>(
            "SELECT team_id, resource_id, role FROM team_members WHERE team_id = ?",
        )
        .bind(team_id)
        .fetch_all(pool)
        .await?)
    }
    /// The (first) team a resource belongs to, for effective-constant resolution (design §3.3.8a).
    pub async fn team_of_resource(
        pool: &SqlitePool,
        resource_id: i64,
    ) -> Result<Option<i64>, DbError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT team_id FROM team_members WHERE resource_id = ? \
             ORDER BY (role = 'lead') DESC, joined_at DESC LIMIT 1",
        )
        .bind(resource_id)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|r| r.0))
    }
}

pub struct TeamOverridesRepo;
impl TeamOverridesRepo {
    pub async fn upsert(pool: &SqlitePool, o: &TeamOverride) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO team_overrides (team_id, pd_hours, pm_workdays, overload_threshold, \
             underload_threshold, utilization_green, utilization_yellow) \
             VALUES (?,?,?,?,?,?,?) \
             ON CONFLICT(team_id) DO UPDATE SET \
             pd_hours=excluded.pd_hours, pm_workdays=excluded.pm_workdays, \
             overload_threshold=excluded.overload_threshold, \
             underload_threshold=excluded.underload_threshold, \
             utilization_green=excluded.utilization_green, \
             utilization_yellow=excluded.utilization_yellow",
        )
        .bind(o.team_id)
        .bind(o.pd_hours)
        .bind(o.pm_workdays)
        .bind(o.overload_threshold)
        .bind(o.underload_threshold)
        .bind(o.utilization_green)
        .bind(o.utilization_yellow)
        .execute(pool)
        .await?;
        Ok(())
    }
    pub async fn get(pool: &SqlitePool, team_id: i64) -> Result<Option<TeamOverride>, DbError> {
        Ok(sqlx::query_as::<_, TeamOverride>(
            "SELECT team_id, pd_hours, pm_workdays, overload_threshold, underload_threshold, \
             utilization_green, utilization_yellow FROM team_overrides WHERE team_id = ?",
        )
        .bind(team_id)
        .fetch_optional(pool)
        .await?)
    }
}
