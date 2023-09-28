
// The web server module
pub mod web_server {
    use ntex::web;
    use serde::{Deserialize, Serialize};
    use home::home_dir;

    #[web::get("/")]
    pub async fn index() -> impl web::Responder {
        format!("Hello World")
    }

    pub fn get_connection() -> sqlite::ConnectionWithFullMutex {
        let home_path = home_dir().unwrap();
        let home_path_str = home_path.as_os_str().to_str().unwrap();
        let pth = home_path_str.to_string() + "\\tm_db";

        let db = sqlite::Connection::open_with_full_mutex(pth + "\\task_data.db").unwrap();
        db
    }

    #[derive(Serialize, Deserialize)]
    pub struct Task {
        pub task_type: String,
        pub name: String,
        pub start_time: i64,
        pub end_time: i64,
        pub is_done: i64,
    }

    use ntex::http::header;
    use ntex::http::Response;
    #[web::post("/get_task")]
    pub async fn list_tasks() -> Response {
        let cnt = get_connection();

        let query = "SELECT * FROM tasks";
        let mut statement = cnt.prepare(query).unwrap();

        let mut res: Vec<Task> = vec![];

        while let Ok(sqlite::State::Row) = statement.next() {
            let new_tsk = Task {
                task_type: statement.read::<String, _>("type").unwrap(),
                name: statement.read::<String, _>("name").unwrap(),
                is_done: statement.read::<i64, _>("is_done").unwrap(),
                start_time: statement.read::<i64, _>("start_time").unwrap(),
                end_time: statement.read::<i64, _>("end_time").unwrap(),
            };
            res.push(new_tsk);
        }

        let j = serde_json::to_string(&res).unwrap();
        Response::Ok()
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .message_body(j.into())
    }

    #[web::get("/get_task")]
    pub async fn list_tasks_get() -> impl web::Responder {
        let cnt = get_connection();

        let query = "SELECT * FROM tasks";
        let mut statement = cnt.prepare(query).unwrap();

        let mut res: Vec<Task> = vec![];

        while let Ok(sqlite::State::Row) = statement.next() {
            let new_tsk = Task {
                task_type: statement.read::<String, _>("type").unwrap(),
                name: statement.read::<String, _>("name").unwrap(),
                is_done: statement.read::<i64, _>("is_done").unwrap(),
                start_time: statement.read::<i64, _>("start_time").unwrap(),
                end_time: statement.read::<i64, _>("end_time").unwrap(),
            };
            res.push(new_tsk);
        }

        let j = serde_json::to_string(&res).unwrap();
        j
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CreatedTask {
        task_type: String,
        name: String,
        start_time: i64,
        end_time: i64,
    }
    #[web::post("/create_task")]
    pub async fn create_task(task_str: String) -> Result<String, web::Error> {
        let cnt = get_connection();

        let task = serde_json::from_str::<CreatedTask>(task_str.as_str()).unwrap();
        let query = format!(
            "INSERT INTO tasks VALUES ('{}', '{}', {}, {}, 0)",
            task.task_type, task.name, task.start_time, task.end_time
        );

        cnt.execute(query).unwrap_or_else(|err| {
            println!("Error : Creating New Task {err}");
            println!("{:#?}", task);
        });

        Ok(String::from("Success"))
    }


    #[derive(Serialize, Deserialize)]
    pub struct UpdateTask {
        name: String,
        is_done: bool,
    }
    #[web::post("/update_task")]
    pub async fn update_task(task_str : String) -> Result<String, web::Error> {
        let cnt = get_connection();

        let task = serde_json::from_str::<UpdateTask>(task_str.as_str()).unwrap();

        let query = format!("SELECT * FROM tasks WHERE name = ?");
        let mut statement = cnt.prepare(query).unwrap();

        statement.bind((1, task.name.as_str())).unwrap();
        if let Ok(sqlite::State::Row) = statement.next() {
            let query = format!(
                "UPDATE tasks SET is_done = {} WHERE name = '{}' AND NOT type = 'F'",
                if task.is_done { "TRUE" } else { "FALSE" },
                task.name
            );
            cnt.execute(query).unwrap();
            Ok(String::from("Success"))
        } else {
            Err(web::error::ErrorNotFound(task.name.clone()).into())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct DeleteTask {
        name: String,
    }
    #[web::post("/delete_task")]
    pub async fn delete_task(task: web::types::Json<DeleteTask>) -> Result<String, web::Error> {
        let cnt = get_connection();

        let query = format!("DELETE FROM tasks WHERE name = '{}'", task.name);

        if let Ok(_) = cnt.execute(query) {
            Ok(String::from("Success"))
        } else {
            Err(web::error::ErrorNotFound(task.name.clone()).into())
        }
    }

    
}



// The task manager Module
pub mod task_manager {
    use std::fs;
    use std::process::exit;
    use std::thread;
    use std::time::Duration;
    
    use chrono;
    use chrono::Datelike;
    use chrono::Timelike;

    use super::web_server::get_connection;
    use super::web_server::Task;

    // Import Google Calendar Tasks 
    use super::google_api::do_call;
    pub async fn gc_import() {
        // Getting the tasks from Google Calendar API integration
        let tasks = do_call().await.unwrap_or_else(|_err| {
            println!("Couldn't connect to Google Calendar API");
            vec![]
        });

        let cnt = get_connection();
        
        for i in tasks {
            // If the task is in Today
            if chrono::DateTime::from_timestamp(i.start_time, 0).unwrap().day0() == chrono::Local::now().day0() {
                let ver_qr = format!("SELECT * FROM tasks WHERE name = '{}'", i.name);
                let mut statement = cnt.prepare(ver_qr).unwrap();

                let mut is_already_created = false;

                while let Ok(sqlite::State::Row) = statement.next() {
                    // If the Task doesn't exist before
                    if statement.read::<i64, _>("start_time").unwrap() == i.start_time && statement.read::<i64, _>("end_time").unwrap() == i.end_time {
                        is_already_created = true;
                    }
                }

                if  !is_already_created {
                    // Push it to the database
                    let query = format!("INSERT INTO tasks VALUES ('{}', '{}', '{}', '{}', 0)", i.task_type, i.name, i.start_time, i.end_time);

                    cnt.execute(query).unwrap_or_else(|err| {
                        println!("Error Creating Task : {}", err);
                    })
                }

            }
        }
    }

    // Update The Database by only keeping today's tasks + priority ones
    use home::home_dir;
    pub async fn update_db() {
        let cnt = get_connection();

        let query = "SELECT * FROM tasks";
        let mut statement = cnt.prepare(query).unwrap();

        let mut res: Vec<Task> = vec![];

        while let Ok(sqlite::State::Row) = statement.next() {
            let new_tsk = Task {
                task_type: statement.read::<String, _>("type").unwrap(),
                name: statement.read::<String, _>("name").unwrap(),
                is_done: statement.read::<i64, _>("is_done").unwrap(),
                start_time: statement.read::<i64, _>("start_time").unwrap(),
                end_time: statement.read::<i64, _>("end_time").unwrap(),
            };
            res.push(new_tsk);
        }
        let home_path = home_dir().unwrap();
        let home_path_str = home_path.as_os_str().to_str().unwrap();
        let pth = home_path_str.to_string() + "\\tm_db";

        for i in res.iter() {
            let dt = chrono::DateTime::from_timestamp(i.start_time, 0).unwrap();
            let sec_dt = chrono::Local::now();

            if i.task_type == "F" && sec_dt.day0() > dt.day0() {
                // Deleting the task
                let query = format!("DELETE FROM tasks WHERE name = '{}'", i.name);

                if let Ok(_) = cnt.execute(query) {
                    let dat = format!("{}:{}:{}:{}", i.name, i.task_type, i.start_time, i.end_time);
                    fs::write(format!("{}\\db_log\\{}.txt", pth, dt.day0()), dat).unwrap();
                } else {
                    exit(-1);
                }
            }
        }
        // Just a check to guarrentee the task manager won't work before the db is updated
        gc_import().await;
        thread::sleep(Duration::from_secs(2));
    }

    // Task Managing function
    pub fn task_manager() {
        let cnt = get_connection();

        let query = "SELECT * FROM tasks";
        let mut statement = cnt.prepare(query).unwrap();

        let mut res: Vec<Task> = vec![];

        while let Ok(sqlite::State::Row) = statement.next() {
            let new_tsk = Task {
                task_type: statement.read::<String, _>("type").unwrap(),
                name: statement.read::<String, _>("name").unwrap(),
                is_done: statement.read::<i64, _>("is_done").unwrap(),
                start_time: statement.read::<i64, _>("start_time").unwrap(),
                end_time: statement.read::<i64, _>("end_time").unwrap(),
            };
            res.push(new_tsk);
        }

        for i in res.iter() {
            let start_dt = chrono::DateTime::from_timestamp(i.start_time, 0).unwrap();
            let end_dt = chrono::DateTime::from_timestamp(i.end_time, 0).unwrap();
            let cur_dt = chrono::Local::now();


            // If you don't do ur tasks :>
            if (start_dt.hour() < cur_dt.hour()
                || (start_dt.hour() == cur_dt.hour() && start_dt.minute() < cur_dt.minute()))
                && (end_dt.hour() > cur_dt.hour()
                    || (end_dt.hour() == cur_dt.hour() && cur_dt.minute() < end_dt.minute()))
            {

                // Haha it shuts down your pc
                std::process::Command::new("shutdown")
                    .arg("-s")
                    .spawn()
                    .unwrap();
            } else {
                // Else it marks it as done
                if i.task_type =="F" && i.is_done == 0 {
                    let query = format!("UPDATE tasks SET is_done=1 WHERE name = '{}' AND start_time = {} AND end_time = {}", 
                                i.name, i.start_time, i.end_time);
                    
                    cnt.execute(query).unwrap_or_else(|err| {
                        println!("Error Updating Database {}", err);
                    })
                }
            }
        }
        // println!("Check Completed. You don't have an ongoing task (for now)");
    }
}


// MOD Google API Integration
pub mod google_api {
    use google_calendar::{Client, types::Error};
    use super::web_server::Task;
    use home::home_dir;

    
    static CLIENT_ID : &str = "YOUR CLIEND ID";
    static CLIENT_SECRET : &str = "YOUR CLIENT SECRET";
    static REDIRECT_URI : &str = "http://localhost";

    


    pub async fn do_call() -> Result<Vec<Task>, Error>{
        let home_path = home_dir().unwrap();
        let home_path_str = home_path.as_os_str().to_str().unwrap();
        let pth = home_path_str.to_string() + "\\tm_db";



        let mut client = Client::new(CLIENT_ID, CLIENT_SECRET, REDIRECT_URI, "", "");

        
        let tk_json = std::fs::read_to_string(pth.clone() + "./token.json").unwrap_or_else(|err| {
            println!("Lost Token : {}", err);
            // Accept and once you are redirected keep the state & code appearing in the url for later use
            let user_consent_url = client.user_consent_url(&["https://www.googleapis.com/auth/calendar.readonly".to_string()]);
    
            println!("{}", user_consent_url);

            std::process::exit(-1);
        });

        // Getting the Token Either from file or from The OAuth
        let mut token: google_calendar::AccessToken;
        token = if tk_json.len() > 1 {
            token = serde_json::from_str::<google_calendar::AccessToken>(&tk_json).unwrap();

            // Refreshing the token
            client = Client::new(CLIENT_ID, CLIENT_SECRET, REDIRECT_URI, token.access_token.clone(), token.refresh_token.clone());

            client.refresh_access_token().await.unwrap();
            token 
        } else {
            // Put the State/Code you spoofed from the URL here
            let state = "___";
            let code = "____";


            token = client.get_access_token(code, state).await.unwrap();

            token
        };

        // Saving the token
        std::fs::write(pth.clone() + "./token.json", serde_json::to_string(&token).unwrap()).unwrap();

        // Listing Calendars
        let event = google_calendar::events::Events {
            client,
        };
        let res = event.list_all("primary", "", 0,
                            google_calendar::types::OrderBy::StartTime, &[], "", &[],
                            false, false, true, "", chrono::Local::now().to_rfc3339().as_str(), "", "").await.unwrap().body;

        let mut ret : Vec<Task> = Vec::new();
        ret.reserve(res.len());

        for i in res {
            ret.push(Task {
                task_type : String::from("F"),
                name : i.summary,
                start_time : i.start.as_ref().unwrap().date_time.unwrap().timestamp() + 3600,
                end_time : i.end.as_ref().unwrap().date_time.unwrap().timestamp() + 3600,
                is_done : 0
            });
        }

        Ok(ret)
    }
}