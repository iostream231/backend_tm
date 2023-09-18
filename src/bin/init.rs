
use std::result::Result;

use sqlite::{Error, State};
use std::time;


fn main() -> Result<(), Error> {
    let connection = sqlite::open("db/task_data.db").unwrap();    
    
    let fst_date = time::SystemTime::now(); // Current time
    let sec_date = fst_date + time::Duration::from_secs_f32(3600.0 * 2.0);  // After 2 hours
    let query = format!("
        CREATE TABLE tasks (type TEXT, name TEXT, start_time INTEGER, end_time INTEGER, is_done INTEGER);
        INSERT INTO tasks VALUES ('A', 'Testing this thing', unixepoch('now'), {}, TRUE);
        INSERT INTO tasks VALUES ('B', 'eat a pizza', unixepoch('now'), {} ,FALSE);
    ", sec_date.duration_since(time::UNIX_EPOCH).unwrap().as_secs(), 
        sec_date.duration_since(time::UNIX_EPOCH).unwrap().as_secs()
    );

    connection.execute(query).unwrap_or_else(|err| {
       println!("Error executing statement : {err}");
     });
 
    
    let query = "SELECT * FROM tasks WHERE type = ?";
    let mut statement = connection.prepare(query)?;
    statement.bind((1, "A")).unwrap();

    while let Ok(State::Row) = statement.next() {
        println!("{} :   {}. is_done: {}. Start time : {}. End time : {}", 
                 statement.read::<String, _>("type").unwrap(),
                 statement.read::<String, _>("name").unwrap(),
                 statement.read::<i64, _>("is_done").unwrap(),
                 statement.read::<i64, _>("start_time").unwrap(),
                 statement.read::<i64, _>("end_time").unwrap()
                 );
    }
    
    

    Ok(())
}
