package main

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/gorilla/mux"
)

// Estructura que representa los datos del estudiante
type Student struct {
	Student    string `json:"student"`
	Age        int    `json:"age"`
	Faculty    string `json:"faculty"`
	Discipline int    `json:"discipline"`
}

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

		// Imprimir los datos del estudiante en la consola (puedes reemplazar esto con cualquier otra lógica)
		fmt.Printf("Estudiante: %s, Edad: %d, Facultad: %s, Disciplina: %d\n",
			student.Student, student.Age, student.Faculty, student.Discipline)

		// Enviar una respuesta exitosa
		w.WriteHeader(http.StatusCreated)
		w.Write([]byte("Estudiante registrado correctamente"))
	}).Methods("POST")

	// Iniciar el servidor
	http.ListenAndServe(":8080", r)
}
