
use std::fs;
use std::process::exit;
use std::thread;
use std::time::Duration;

use chrono::Datelike;
use chrono::Timelike;
use serde::{Serialize, Deserialize};
use ntex::web;
use chrono;

#[web::get("/")]
async fn index() -> impl web::Responder {
    format!("Hello World")    
}


fn get_connection() -> sqlite::ConnectionWithFullMutex {
    let db = sqlite::Connection::open_with_full_mutex("db/task_data.db").unwrap();
    db
}



#[derive(Serialize, Deserialize)]
struct Task {
    task_type: String,
    name : String,
    start_time : i64,
    end_time : i64,
    is_done : i64
}

use ntex::http::Response;
use ntex::http::header;
#[web::post("/get_task")]
async fn list_tasks() -> Response {
    let cnt = get_connection(); 
    
    let query = "SELECT * FROM tasks";
    let mut statement = cnt.prepare(query).unwrap();

   
    let mut res : Vec<Task> = vec![];
 
    while let Ok(sqlite::State::Row) = statement.next() {
        let new_tsk =Task { 
            task_type : statement.read::<String, _>("type").unwrap(),
            name : statement.read::<String, _>("name").unwrap(),
            is_done : statement.read::<i64, _>("is_done").unwrap(),
            start_time : statement.read::<i64, _>("start_time").unwrap(),
            end_time : statement.read::<i64, _>("end_time").unwrap()
        }; 
        res.push(new_tsk);
    }


    let j = serde_json::to_string(&res).unwrap();
    Response::Ok()
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .message_body(j.into())
}


#[web::get("/get_task")]
async fn list_tasks_get() -> impl web::Responder {
    let cnt = get_connection(); 
    
    let query = "SELECT * FROM tasks";
    let mut statement = cnt.prepare(query).unwrap();

   
    let mut res : Vec<Task> = vec![];
 
    while let Ok(sqlite::State::Row) = statement.next() {
        let new_tsk =Task { 
            task_type : statement.read::<String, _>("type").unwrap(),
            name : statement.read::<String, _>("name").unwrap(),
            is_done : statement.read::<i64, _>("is_done").unwrap(),
            start_time : statement.read::<i64, _>("start_time").unwrap(),
            end_time : statement.read::<i64, _>("end_time").unwrap()

        }; 
        res.push(new_tsk);
    }


    let j = serde_json::to_string(&res).unwrap();
    j 
}




#[derive(Serialize, Deserialize, Debug)]
struct CreatedTask {
    task_type : String, 
    name : String,
    start_time : i64,
    end_time: i64
}
#[web::post("/create_task")]
async fn create_task(task_str : String) -> Result<String, web::Error> {
    let cnt = get_connection();

    let task = serde_json::from_str::<CreatedTask>(task_str.as_str()).unwrap();
    let query = format!("INSERT INTO tasks VALUES ('{}', '{}', {}, {}, 0)", task.task_type, task.name, task.start_time, task.end_time);

    cnt.execute(query).unwrap_or_else(|err| {
        println!("Error : Creating New Task {err}");
        println!("{:#?}", task);
    });

    Ok(String::from("Success"))
}

#[web::post("test_something")]
async fn test_something(task : String) -> Result<String, web::Error> {
    println!("{}", task);

    Ok(String::from("Success"))
}


#[derive(Serialize, Deserialize)]
struct UpdateTask {
    name : String,
    is_done : bool,
}
#[web::post("/update_task")]
async fn update_task(task: web::types::Json<UpdateTask>) -> Result<String, web::Error> {
    let cnt = get_connection();

    let query = format!("SELECT * FROM tasks WHERE name = ?");
    let mut statement = cnt.prepare(query).unwrap();

    statement.bind((1, task.name.as_str())).unwrap();
    if let Ok(sqlite::State::Row) = statement.next() {
        let query = format!("UPDATE tasks SET is_done = {} WHERE name = '{}'", if task.is_done { "TRUE" } else { "FALSE" }, task.name);
        cnt.execute(query).unwrap();
        Ok(String::from("Success"))
    } else {
        Err(web::error::ErrorNotFound(task.name.clone()).into())
    }

}


#[derive(Serialize, Deserialize)]
struct DeleteTask {
    name : String
}
#[web::post("/delete_task")]
async fn delete_task(task: web::types::Json<DeleteTask>) -> Result<String, web::Error> {
    let cnt = get_connection();

    let query = format!("DELETE FROM tasks WHERE name = '{}'", task.name);
    
    if let Ok(_) = cnt.execute(query) {
        Ok(String::from("Success"))
    } else {
        Err(web::error::ErrorNotFound(task.name.clone()).into())
    }

}



// Update The Database by only keeping today's tasks + priority ones
fn update_db() {
    let cnt = get_connection(); 
    
    let query = "SELECT * FROM tasks";
    let mut statement = cnt.prepare(query).unwrap();

   
    let mut res : Vec<Task> = vec![];
 
    while let Ok(sqlite::State::Row) = statement.next() {
        let new_tsk =Task { 
            task_type : statement.read::<String, _>("type").unwrap(),
            name : statement.read::<String, _>("name").unwrap(),
            is_done : statement.read::<i64, _>("is_done").unwrap(),
            start_time : statement.read::<i64, _>("start_time").unwrap(),
            end_time : statement.read::<i64, _>("end_time").unwrap()

        }; 
        res.push(new_tsk);
    }

    for i in res.iter() {
        let dt = chrono::DateTime::from_timestamp(i.start_time, 0).unwrap();
        let sec_dt = chrono::Local::now();

        if i.task_type == "F" && sec_dt.day0() > dt.day0() {

            // Deleting the task
            let query = format!("DELETE FROM tasks WHERE name = '{}'", i.name);

            if let Ok(_) = cnt.execute(query) {
                let dat = format!("{}:{}:{}:{}", i.name, i.task_type, i.start_time, i.end_time);
                fs::write(format!("db/{}.txt", dt.day0()), dat).unwrap();
                println!("Deleted Task {}", i.name);
            } else {
                exit(-1);
            }
            
        }
    }
}


// Task Managing function
async fn task_manager() {
    thread::sleep(Duration::from_secs(5 * 60));

    let cnt = get_connection(); 
    
    let query = "SELECT * FROM tasks";
    let mut statement = cnt.prepare(query).unwrap();

   
    let mut res : Vec<Task> = vec![];
 
    while let Ok(sqlite::State::Row) = statement.next() {
        let new_tsk =Task { 
            task_type : statement.read::<String, _>("type").unwrap(),
            name : statement.read::<String, _>("name").unwrap(),
            is_done : statement.read::<i64, _>("is_done").unwrap(),
            start_time : statement.read::<i64, _>("start_time").unwrap(),
            end_time : statement.read::<i64, _>("end_time").unwrap()

        }; 
        res.push(new_tsk);
    }
    

    for i in res.iter() {
        let start_dt = chrono::DateTime::from_timestamp(i.start_time, 0).unwrap();
        let end_dt = chrono::DateTime::from_timestamp(i.end_time, 0).unwrap();
        let cur_dt = chrono::Local::now();

        // If you don't do ur tasks :>
        if start_dt.hour() <= cur_dt.hour() && (end_dt.hour() >= cur_dt.hour() || end_dt.minute() >= cur_dt.minute()) {
            // Haha it shuts down your pc
            std::process::Command::new("program").arg("/s");
        }
    }

}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    // Updating the Database
    update_db();

    // Creating an idle thread for the task manager
    let th = thread::spawn(|| {
        // Works indefinitely ( this is seems like a bad choice ngl )
        while true {
            task_manager();
            thread::sleep(Duration::from_secs(2 * 60)); // Wait another 2 minutes ( just in case );
        }
    });


    web::HttpServer::new(move || {
        web::App::new()    
            .service(index)
            .service(list_tasks)
            .service(list_tasks_get)
            .service(create_task)
            .service(update_task)
            .service(delete_task)
    })
    .bind(("127.0.0.1", 3050))?
    .run()
    .await
    
}
