use crate::error::AppError;
use chrono::NaiveDate;

pub(crate) struct RequiredWindow {
    pub start: String,
    pub end: String,
}

#[tracing::instrument(fields(start = %start, end = %end))]
pub(crate) fn required_window(start: &str, end: &str) -> Result<RequiredWindow, AppError> {
    let start_date = parse_required(start)?;
    let end_date = parse_required(end)?;
    if end_date < start_date {
        tracing::warn!(start = %start, end = %end, "invalid date window");
        return Err(domain::DomainError::InvalidDateWindow.into());
    }
    Ok(RequiredWindow {
        start: format_date(start_date),
        end: format_date(end_date),
    })
}

#[tracing::instrument(fields(value = %value))]
fn parse_required(value: &str) -> Result<NaiveDate, AppError> {
    if value.len() != 10 {
        tracing::warn!(value = %value, "invalid date length");
        return Err(domain::DomainError::InvalidDateWindow.into());
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| {
            tracing::warn!(value = %value, "invalid date format");
            domain::DomainError::InvalidDateWindow
        })
        .map_err(Into::into)
}

fn format_date(value: NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}
