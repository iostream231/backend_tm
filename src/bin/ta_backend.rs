#![windows_subsystem = "windows"]

use backend::web_server::*;
use backend::task_manager::{update_db, task_manager};

use std::thread;
use std::time::Duration;
use ntex::web;





#[ntex::main]
async fn main() -> std::io::Result<()> {


    // Updating the Database
    println!("Database Updated");
    update_db().await;

    // Creating an idle thread for the task manager
    let _th = thread::spawn(|| {
        // Works indefinitely ( this is seems like a bad choice ngl )
        loop {
            task_manager();
            println!("All Tasks Checked.");
            thread::sleep(Duration::from_secs(5 * 60));
            // Waits 5 mins before the next check
        }
    });



    // Running the web server
    web::HttpServer::new(move || {
        println!("Listening at 3050");
        web::App::new()    
            .service(index)
            .service(list_tasks)
            .service(list_tasks_get)
            .service(create_task)
            .service(update_task)
            .service(delete_task)
    })
    .workers(1)
    .bind(("127.0.0.1", 3050))?
    .run()
    .await

    
    
}
