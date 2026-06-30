// Shared date helpers. The whole app stores dates as `YYYY-MM-DD` strings and feeds epoch-ms
// timestamps to Naive UI date pickers (which operate in the LOCAL timezone). These two helpers
// are exact inverses in local time, so a date round-trips (backend string → picker → backend
// string) without the off-by-one shift you get from `Date.parse("YYYY-MM-DD")`, which parses a
// date-only string as UTC midnight and lands on the previous day in zones west of UTC.

/** Format a local epoch-ms timestamp as `YYYY-MM-DD` using LOCAL calendar components. */
export function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

/** Like {@link fmtDate} but passes `null`/`undefined` through (for optional date fields). */
export function fmtDateOrNull(ms: number | null | undefined): string | null {
  return ms == null ? null : fmtDate(ms);
}

/** Parse a `YYYY-MM-DD` string to a LOCAL-midnight epoch-ms — the inverse of {@link fmtDate}. */
export function parseDate(s: string | null | undefined): number | null {
  if (!s) return null;
  const [y, m, d] = s.split("-").map(Number);
  if (!y || !m || !d) return null;
  return new Date(y, m - 1, d).getTime();
}

/** Parse a required `YYYY-MM-DD` string to local-midnight epoch-ms (throws on bad input). */
export function parseDateStrict(s: string): number {
  const ms = parseDate(s);
  if (ms == null) throw new Error(`invalid date: ${s}`);
  return ms;
}
