package main

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/gorilla/mux"

	"context"
	"log"

	pb "agronomia/Disciplicas/agronomia/proto"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// Estructura que representa los datos del estudiante
type Student struct {
	Student    string `json:"name"`
	Age        int32  `json:"age"`
	Faculty    string `json:"faculty"`
	Discipline int32  `json:"discipline"`
}

// Definir el servidor
// type server struct{}

// func (s *server) SendStudent(ctx context.Context, in *pb.Student) (*pb.Status, error) {
//     // Aquí procesarás el estudiante recibido
//     log.Printf("Recibido: %v", in)

//     // Simulando una respuesta exitosa
//     return &pb.Status{Success: true, Message: "Estudiante recibido correctamente"}, nil
// }

func main() {
	r := mux.NewRouter()

	// Ruta para recibir las peticiones POST
	r.HandleFunc("/students", func(w http.ResponseWriter, r *http.Request) {
		// Decodificar el cuerpo de la petición JSON
		var student Student
		err := json.NewDecoder(r.Body).Decode(&student)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		//INtento por enviar con gR
		// Crear un cliente gRPC para conectarse al servidor
		conn, err := grpc.Dial("localhost:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		defer conn.Close()
		client := pb.NewStudentServiceClient(conn)

		protoStudent := pb.Student{
			Name:       student.Student,
			Age:        student.Age,
			Faculty:    student.Faculty,
			Discipline: student.Discipline,
		}

		//Enviar
		_, err = client.SendStudent(context.Background(), &protoStudent)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		fmt.Println("Estudiante enviado correctamente a gRPC")
		w.WriteHeader(http.StatusCreated)
		w.Write([]byte("Estudiante registrado correctamente"))
		// Enviar una respuesta exitosa
	}).Methods("POST")
	log.Fatal(http.ListenAndServe(":8080", r))
}
