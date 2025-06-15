package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"math/rand"
	"net"
	"time"

	pb "go-server/proto"

	"github.com/IBM/sarama" // Importar librería Kafka
	"google.golang.org/grpc"
)

// 50051 = Natacion
// 50052 = Atletismo
// 50053 = Boxeo
var (
	port      = flag.Int("port", 50051, "The server port")
	kafkaAddr = "my-cluster-kafka-bootstrap:9092" // Dirección del servicio de Kafka en Kubernetes
)

type StudentData struct {
	Faculty    string `json:"faculty"`
	Discipline string `json:"discipline"`
	Name       string `json:"name"`
}

// Server is used to implement the gRPC server in the proto library
type server struct {
	pb.UnimplementedStudentServer
	kafkaProducer sarama.SyncProducer
}

// Implement the GetStudent method
func (s *server) GetStudent(_ context.Context, in *pb.StudentRequest) (*pb.StudentResponse, error) {
	log.Printf("Received: %v", in)

	// Algoritmo para determinar ganador o perdedor
	isWinner := s.determineWinner()

	// Configurar el mensaje para enviar a Kafka
	topic := "winners"
	if !isWinner {
		topic = "losers"
	}

	data := StudentData{
		Faculty:    in.GetFaculty(),
		Discipline: in.GetDiscipline().String(),
		Name:       in.GetName(),
	}

	jsonData, err1 := json.Marshal(data)
	if err1 != nil {
		panic(err1)
	}

	message := &sarama.ProducerMessage{
		Topic: topic,
		Value: sarama.StringEncoder(jsonData),
	}

	// Enviar el mensaje a Kafka
	_, _, err := s.kafkaProducer.SendMessage(message)
	if err != nil {
		log.Printf("Error sending message to Kafka: %v", err)
		return &pb.StudentResponse{Success: false}, err
	}

	return &pb.StudentResponse{Success: true}, nil
}

// Método para determinar ganador o perdedor
func (s *server) determineWinner() bool {
	rand.Seed(time.Now().UnixNano())
	return rand.Intn(2) == 0 // 50% probabilidad de ganar (0 = ganador, 1 = perdedor)
}

func main() {
	flag.Parse()

	// Configurar el productor de Kafka
	config := sarama.NewConfig()
	config.Producer.Return.Successes = true
	producer, err := sarama.NewSyncProducer([]string{kafkaAddr}, config)
	if err != nil {
		log.Fatalf("Failed to create Kafka producer: %v", err)
	}
	defer producer.Close()

	// Configurar el servidor de gRPC
	lis, err := net.Listen("tcp", fmt.Sprintf(":%d", *port))
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}
	s := grpc.NewServer()
	pb.RegisterStudentServer(s, &server{kafkaProducer: producer})

	log.Printf("Server started on port %d", *port)
	if err := s.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
