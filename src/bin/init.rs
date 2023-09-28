
use std::result::Result;
use sqlite::Error;
use home::home_dir;

fn main() -> Result<(), Error> {
    let home_path = home_dir().unwrap();
    let home_path_str = home_path.as_os_str().to_str().unwrap();
    let pth = home_path_str.to_string() + "\\tm_db";

    // std::fs::create_dir(home_path_str.to_string() + "\\tm_db").unwrap();
    std::fs::write(pth.clone() + "\\task_data.db", "").unwrap();


    
    

    let connection = sqlite::open(pth.clone() + "\\task_data.db").unwrap();    
    
    // let query = format!("
    //     CREATE TABLE tasks (type TEXT, name TEXT, start_time INTEGER, end_time INTEGER, is_done INTEGER);
    //     INSERT INTO tasks VALUES ('A', 'Testing this thing', unixepoch('now'), {}, TRUE);
    //     INSERT INTO tasks VALUES ('B', 'eat a pizza', unixepoch('now'), {} ,FALSE);
    // ", sec_date.duration_since(time::UNIX_EPOCH).unwrap().as_secs(), 
    //     sec_date.duration_since(time::UNIX_EPOCH).unwrap().as_secs()
    // );

    let query = "CREATE TABLE tasks (type TEXT, name TEXT, start_time INTEGER, end_time INTEGER, is_done INTEGER)";

    connection.execute(query).unwrap_or_else(|err| {
       println!("Error executing statement : {err}");
     });
 
    
    // let query = "SELECT * FROM tasks WHERE type = ?";
    // let mut statement = connection.prepare(query)?;
    // statement.bind((1, "A")).unwrap();

    // while let Ok(State::Row) = statement.next() {
    //     println!("{} :   {}. is_done: {}. Start time : {}. End time : {}", 
    //              statement.read::<String, _>("type").unwrap(),
    //              statement.read::<String, _>("name").unwrap(),
    //              statement.read::<i64, _>("is_done").unwrap(),
    //              statement.read::<i64, _>("start_time").unwrap(),
    //              statement.read::<i64, _>("end_time").unwrap()
    //              );
    // }
    
    

    Ok(())
}
