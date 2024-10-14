package main

import (
	"context"
	"log"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"

	pb "agronomia/Disciplicas/agronomia/proto" // Reemplaza "tu_proyecto/proto" con la ruta correcta a tu archivo .proto
)

type server struct {
	pb.UnimplementedStudentServiceServer
}

func (s *server) SendStudent(ctx context.Context, in *pb.Student) (*pb.Status, error) {
	log.Printf("Recibido: %v", in)
	return &pb.Status{Success: true, Message: "Estudiante recibido correctamente"}, nil
}

func main() {
	lis, err := net.Listen("tcp", ":50051")
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}
	s := grpc.NewServer()
	pb.RegisterStudentServiceServer(s, &server{})
	reflection.Register(s)
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
