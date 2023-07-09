use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute(
        "CREATE VIRTUAL TABLE email USING fts5(sender, title, body);",
        (), // empty list of parameters.
    )?;

    conn.execute(
        "INSERT INTO email (sender, title, body) VALUES (?1, ?2, ?3)",
        (
            "strrl",
            "hi I am trying sqlite3 with fts5 in rust",
            "no body",
        ),
    )?;

    let mut stmt = conn.prepare("SELECT * FROM email WHERE title MATCH 'rust OR fts OR sqlite';")?;
    let iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    for item in iter {
        println!("{:?}", item)
    }
    Ok(())
}
