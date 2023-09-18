
use serde::{Serialize, Deserialize};
use ntex::web;

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




#[derive(Serialize, Deserialize)]
struct CreatedTask {
    task_type : String, 
    name : String,
    start_time : i64,
    end_time: i64
}
#[web::post("/create_task")]
async fn create_task(task : web::types::Form<CreatedTask>) -> Result<String, web::Error> {
    let cnt = get_connection();

    let query = format!("INSERT INTO tasks VALUES ('{}', '{}', {}, {}, 0)", task.task_type, task.name, task.start_time, task.end_time);

    cnt.execute(query).unwrap_or_else(|err| {
        println!("Error : Creating New Task {err}");
    });

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





#[ntex::main]
async fn main() -> std::io::Result<()> {
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
