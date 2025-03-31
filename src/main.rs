use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// Define our data structures
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    id: u64,
    title: String,
    completed: bool,
}

// Our application state
struct AppState {
    tasks: Mutex<Vec<Task>>,
}

// Handler functions for our API endpoints
async fn get_tasks(data: web::Data<AppState>) -> impl Responder {
    let tasks = data.tasks.lock().unwrap();
    HttpResponse::Ok().json(&*tasks)
}

async fn get_task_by_id(path: web::Path<u64>, data: web::Data<AppState>) -> impl Responder {
    let task_id = path.into_inner();
    let tasks = data.tasks.lock().unwrap();

    if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
        HttpResponse::Ok().json(task)
    } else {
        HttpResponse::NotFound().body(format!("Task with ID {} not found", task_id))
    }
}

#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    title: String,
}

async fn create_task(
    data: web::Data<AppState>,
    req: web::Json<CreateTaskRequest>,
) -> impl Responder {
    let mut tasks = data.tasks.lock().unwrap();

    // Generate a new ID (in a real app, you'd use a better ID strategy)
    let new_id = tasks.len() as u64 + 1;

    let new_task = Task {
        id: new_id,
        title: req.title.clone(),
        completed: false,
    };

    tasks.push(new_task.clone());
    HttpResponse::Created().json(new_task)
}

async fn update_task(
    path: web::Path<u64>,
    data: web::Data<AppState>,
    req: web::Json<Task>,
) -> impl Responder {
    let task_id = path.into_inner();
    let mut tasks = data.tasks.lock().unwrap();

    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        // Update task fields
        task.title = req.title.clone();
        task.completed = req.completed;
        HttpResponse::Ok().json(task)
    } else {
        HttpResponse::NotFound().body(format!("Task with ID {} not found", task_id))
    }
}

async fn delete_task(path: web::Path<u64>, data: web::Data<AppState>) -> impl Responder {
    let task_id = path.into_inner();
    let mut tasks = data.tasks.lock().unwrap();

    let initial_len = tasks.len();
    tasks.retain(|t| t.id != task_id);

    if tasks.len() < initial_len {
        HttpResponse::Ok().body(format!("Task with ID {} has been deleted", task_id))
    } else {
        HttpResponse::NotFound().body(format!("Task with ID {} not found", task_id))
    }
}

// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("API is healthy!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://localhost:8080");

    // Initialize application state with some sample data
    let app_state = web::Data::new(AppState {
        tasks: Mutex::new(vec![
            Task {
                id: 1,
                title: "Learn Rust".to_string(),
                completed: false,
            },
            Task {
                id: 2,
                title: "Build an API".to_string(),
                completed: false,
            },
        ]),
    });

    // Set up and start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Register our API routes
            .route("/health", web::get().to(health_check))
            .route("/tasks", web::get().to(get_tasks))
            .route("/tasks", web::post().to(create_task))
            .route("/tasks/{id}", web::get().to(get_task_by_id))
            .route("/tasks/{id}", web::put().to(update_task))
            .route("/tasks/{id}", web::delete().to(delete_task))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
