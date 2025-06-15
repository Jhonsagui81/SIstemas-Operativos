package main

import (
	"context"
	pb "go-agronomia/proto"
	"log"
	"time"

	"github.com/gofiber/fiber/v2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// variable que hace referenica a donde vamos a enviar el gRCP
//
// go-server-service:50051
//
//localhost:50051
// var (
// 	addr = flag.String("addr", "go-server-service:50051", "the address to connect to")
// )

type Student struct {
	Name       string `json:"name"`
	Age        int    `json:"age"`
	Faculty    string `json:"faculty"`
	Discipline int    `json:"discipline"`
}

func sendData(fiberCtx *fiber.Ctx) error {
	var body Student
	if err := fiberCtx.BodyParser(&body); err != nil {
		return fiberCtx.Status(400).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	//Definir la direccion del servidor
	var grpcAddr string
	switch body.Discipline {
	case 0: //Natacion
		// grpcAddr = "localhost:50051"
		grpcAddr = "swimming-service:50051"
	case 2: //Atletismo
		// grpcAddr = "localhost:50052"
		grpcAddr = "athletics-service:50051"
	case 1: //Boxeo
		// grpcAddr = "localhost:50053"
		grpcAddr = "boxing-service:50051"
	default:
		return fiberCtx.Status(400).JSON(fiber.Map{
			"error": "Disciplina inválida",
		})
	}

	// Set up a connection to the server.
	conn, err := grpc.Dial(grpcAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("did not connect: %v", err)
	}
	defer conn.Close()
	//instancia de un cliente estudiane -> se le pasa coneccion conn
	c := pb.NewStudentClient(conn)

	// Create a channel to receive the response and error
	responseChan := make(chan *pb.StudentResponse)
	errorChan := make(chan error)
	go func() { //funcion anonima

		// Contact the server and print out its response.
		ctx, cancel := context.WithTimeout(context.Background(), time.Second)
		defer cancel()

		//madanr estructura
		r, err := c.GetStudent(ctx, &pb.StudentRequest{
			Name:       body.Name,
			Age:        int32(body.Age),
			Faculty:    body.Faculty,
			Discipline: pb.Discipline(body.Discipline),
		})

		if err != nil {
			errorChan <- err
			return
		}

		responseChan <- r
	}() //-> llama funcion anonima

	select {
	case response := <-responseChan:
		return fiberCtx.JSON(fiber.Map{
			"message": response.GetSuccess(),
		})
	case err := <-errorChan:
		return fiberCtx.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	case <-time.After(5 * time.Second):
		return fiberCtx.Status(500).JSON(fiber.Map{
			"error": "timeout",
		})
	}
}

func main() {
	app := fiber.New()
	app.Post("/grpc-go", sendData)

	err := app.Listen(":8080")
	if err != nil {
		log.Println(err)
		return
	}
}
