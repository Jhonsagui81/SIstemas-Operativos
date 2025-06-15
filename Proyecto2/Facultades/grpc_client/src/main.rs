use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use studentgrpc::student_client::StudentClient;
use studentgrpc::StudentRequest;
use tokio::sync::oneshot;
use tokio::task;

pub mod studentgrpc {
    tonic::include_proto!("confproto");
}

#[derive(Deserialize, Serialize)]
struct StudentData {
    name: String,
    age: i32,
    faculty: String,
    discipline: i32,
}

async fn handle_student(student: web::Json<StudentData>) -> impl Responder {
    //posibles servidores
    let grpc_addr = match student.discipline {
        0 => "http://swimming-service:50051",  // Natación
        2 => "http://athletics-service:50051",  // Atletismo
        1 => "http://boxing-service:50051",     // Boxeo
        _ => {
            return HttpResponse::BadRequest().body("Disciplina inválida");
        }
    };

    // Crear un canal oneshot para recibir el resultado de la tarea
    let (response_tx, response_rx) = oneshot::channel();

    // Spawn para manejar la solicitud en un hilo separado
    task::spawn(async move {
        // Crear un cliente gRPC
        let mut client = match StudentClient::connect(grpc_addr.to_string()).await {
            Ok(client) => client,
            Err(e) => {
                let _ = response_tx.send(Err(format!("Failed to connect to gRPC server: {}", e)));
                return;
            },
        };

        // Crear la solicitud gRPC
        let request = tonic::Request::new(StudentRequest {
            name: student.name.clone(),
            age: student.age,
            faculty: student.faculty.clone(),
            discipline: student.discipline,
        });

        // Hacer la llamada gRPC
        match client.get_student(request).await {
            Ok(response) => {
                // Enviar la respuesta por el canal
                let _ = response_tx.send(Ok(response.into_inner().success));
            },
            Err(e) => {
                // Enviar el error por el canal
                let _ = response_tx.send(Err(format!("gRPC call failed: {}", e)));
            }
        }
    });

    // Esperar la respuesta de la tarea
    match response_rx.await {
        Ok(Ok(success)) => HttpResponse::Ok().json(format!("Operation success: {}", success)),
        Ok(Err(err)) => HttpResponse::InternalServerError().body(err),
        Err(_) => HttpResponse::InternalServerError().body("Failed to receive response"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://localhost:8081");
    HttpServer::new(|| {
        App::new()
            .route("/grpc-rust", web::post().to(handle_student))
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
