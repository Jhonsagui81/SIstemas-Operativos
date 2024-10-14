
use actix_web::{web, App, HttpServer, HttpResponse};
use serde:: {Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct Student {
    student: String,
    age: u8,
    faculty: String,
    discipline: u8,
}

async fn create_student(student: web::Json<Student>) -> HttpResponse {
    println!("Received student: {:?}", student);
    println!("Received student: {}", json!(student)); // Imprimimos el JSON


    // Aquí puedes realizar acciones con la información del estudiante,
    // como almacenarla en una base de datos o enviarla a otro servicio.

    HttpResponse::Ok().body("Student created")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/students", web::post().to(create_student))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}