use rusqlite::{Connection, Result};

pub fn init_schema(conn: &Connection) -> Result<()> {
    create_category_table(conn)?;
    create_tag_table(conn)?;
    create_buy_table(conn)?;
    create_bike_table(conn)?;
    create_ride_table(conn)?;
    create_chain_lubrication_table(conn)?;
    create_tag_to_ride_table(conn)?;
    create_tag_to_buy_table(conn)?;
    create_buy_to_bike_table(conn)?;
    create_buy_to_category_table(conn)?;
    Ok(())
}

fn create_category_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS category(
            id      INTEGER PRIMARY KEY,
            abbr    TEXT NOT NULL UNIQUE,
            name    TEXT NOT NULL UNIQUE
        )",
        [],
    )?;
    Ok(())
}

fn create_tag_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tag(
            id      INTEGER PRIMARY KEY,
            name    TEXT NOT NULL UNIQUE
        )",
        [],
    )?;
    Ok(())
}

fn create_buy_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS buy(
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            price       REAL NOT NULL,
            datestamp   NUMERIC NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn create_bike_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bike(
            id              INTEGER PRIMARY KEY,
            category_id     INTEGER NOT NULL,
            id_in_cat       INTEGER NOT NULL,
            name            TEXT UNIQUE NOT NULL,
            datestamp       NUMERIC NOT NULL,
            FOREIGN KEY(category_id) REFERENCES category(id) ON DELETE RESTRICT 
        )",
        [],
    )?;
    Ok(())
}

fn create_ride_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ride(
            id          INTEGER PRIMARY KEY,
            bike_id     INTEGER NOT NULL,
            datestamp   NUMERIC NOT NULL,
            distance    REAL NOT NULL,
            annotation  TEXT,
            FOREIGN KEY(bike_id) REFERENCES bike(id) ON DELETE RESTRICT
        )",
        [],
    )?;
    Ok(())
}

fn create_chain_lubrication_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chain_lubrication(
            id          INTEGER PRIMARY KEY,
            bike_id     INTEGER NOT NULL,
            datestamp   NUMERIC NOT NULL,
            distance    REAL NOT NULL,
            annotation  TEXT,
            FOREIGN KEY(bike_id) REFERENCES bike(id) ON DELETE RESTRICT
        )",
        [],
    )?;
    Ok(())
}

fn create_tag_to_ride_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tag_to_ride(
            id          INTEGER PRIMARY KEY,
            tag_id      INTEGER NOT NULL,
            ride_id     INTEGER NOT NULL,
            FOREIGN KEY(tag_id) REFERENCES tag(id) ON DELETE CASCADE,
            FOREIGN KEY(ride_id) REFERENCES ride(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

fn create_tag_to_buy_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tag_to_buy(
            id          INTEGER PRIMARY KEY,
            tag_id      INTEGER NOT NULL,
            buy_id      INTEGER NOT NULL,
            FOREIGN KEY(tag_id) REFERENCES tag(id) ON DELETE CASCADE,
            FOREIGN KEY(buy_id) REFERENCES buy(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

fn create_buy_to_bike_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS buy_to_bike(
            id          INTEGER PRIMARY KEY,
            buy_id      INTEGER NOT NULL,
            bike_id     INTEGER NOT NULL,
            FOREIGN KEY(buy_id) REFERENCES buy(id) ON DELETE CASCADE,
            FOREIGN KEY(bike_id) REFERENCES bike(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

fn create_buy_to_category_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS buy_to_category(
            id              INTEGER PRIMARY KEY,
            buy_id          INTEGER NOT NULL,
            category_id     INTEGER NOT NULL,
            FOREIGN KEY(buy_id) REFERENCES buy(id) ON DELETE CASCADE,
            FOREIGN KEY(category_id) REFERENCES category(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}
