use crate::common::{Error, Res};
use chrono::NaiveDateTime;
use once_cell::sync::Lazy;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::FromSql;
use rusqlite::{Params, Row, ToSql};

pub(super) fn cast<E>(res: Res<E>) -> rusqlite::Result<E> {
    res.map_err(|e| {
        match e {
            Error::SqliteError(e) => e,
            _ => rusqlite::Error::InvalidColumnName(e.to_string()),
        }
    })
}

pub(super) struct RowData<'tmp> {
    row: &'tmp Row<'tmp>,
}

impl RowData<'_> {
    pub(super) fn get<T: FromSql>(&self, idx: usize) -> Res<T> {
        self.row.get(idx).map_err(|e| Error::SqliteError(e))
    }

    pub(super) fn get_timestamp(&self, idx: usize) -> Res<i64> {
        let datetime_str: String = self.row.get(idx).map_err(|e| Error::SqliteError(e))?;
        let naive_datetime = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S").map_err(|e| Error::CastDbValueError(e.to_string()))?;
        Ok(naive_datetime.and_utc().timestamp())
    }
}

pub(super) fn cast_placeholder<T>(vec: &Vec<T>) -> String {
    vec.iter().map(|_| "?").collect::<Vec<&str>>().join(",")
}

pub(super) fn cast_list<T: ToSql>(vec: &Vec<T>) -> Vec<&dyn ToSql> {
    vec.iter().map(|x| x as &dyn ToSql).collect::<Vec<&dyn ToSql>>()
}

pub(super) const POOL: Lazy<Pool<SqliteConnectionManager>> = Lazy::new(|| {
    let manager = SqliteConnectionManager::file("test/data.db");
    Pool::new(manager).unwrap()
});

pub(super) fn con() -> Res<PooledConnection<SqliteConnectionManager>> {
    POOL.get().map_err(
        |_| Error::Busy("lock fail".to_string())
    )
}

pub(super) fn insert<P: Params>(sql: &str, params: P) -> Res<i64> {
    let connection = con()?;
    let result = match connection.prepare(sql) {
        Ok(mut stat) =>
            match stat.execute(params) {
                Ok(_) => Ok(connection.last_insert_rowid()),
                Err(e) => Err(Error::ConnectError(e.to_string()))
            },
        Err(e) => Err(Error::ConnectError(e.to_string()))
    };
    result
}

pub(super) fn exec<P: Params>(sql: &str, params: P) -> Res<usize> {
    match con()?.prepare(sql) {
        Ok(mut stat) =>
            match stat.execute(params) {
                Ok(res) => Ok(res),
                Err(e) => Err(Error::ConnectError(e.to_string()))
            },
        Err(e) => Err(Error::ConnectError(e.to_string()))
    }
}

pub(super) fn query_one<T, P, F>(sql: &str, params: P, f: F) -> Res<T>
where
    P: Params,
    F: FnOnce(&RowData<'_>) -> Res<T>,
{
    match con()?.prepare(sql) {
        Ok(mut stat) =>
            match stat.query_row(params, |x| cast(f(&RowData { row: x }))) {
                Ok(res) => Ok(res),
                Err(e) => Err(Error::ConnectError(e.to_string()))
            }
        Err(e) => Err(Error::ConnectError(e.to_string()))
    }
}

pub(super) fn query_all<T, P, F>(sql: &str, params: P, mut f: F) -> Res<Vec<T>>
where
    P: Params,
    F: FnMut(&RowData<'_>) -> Res<T>,
{
    match con()?.prepare(sql) {
        Ok(mut stat) =>
            match stat.query_map(params, |x| cast(f(&RowData { row: x }))) {
                Ok(res) => Ok(res.filter(|r| r.is_ok()).map(|r| r.unwrap()).collect()),
                Err(e) => Err(Error::ConnectError(e.to_string()))
            },
        Err(e) => Err(Error::ConnectError(e.to_string()))
    }
}

pub(super) fn update_check(res: Res<usize>, err: Error) -> Res<()> {
    match res {
        Ok(cnt) => if cnt == 1 { Ok(()) } else { Err(err) },
        Err(e) => Err(e)
    }
}

pub(super) fn map_id(row: &RowData<'_>) -> Res<i64> {
    row.get(0)
}

pub(super) fn map_count(row: &RowData<'_>) -> Res<usize> {
    row.get(0)
}
