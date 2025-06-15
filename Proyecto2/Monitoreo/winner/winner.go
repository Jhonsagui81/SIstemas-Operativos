package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"github.com/IBM/sarama"
	"github.com/go-redis/redis/v8"
)

var (
	kafkaBrokers = []string{"my-cluster-kafka-bootstrap:9092"}
	topic        = "losers"
	redisClient  *redis.Client
)

type StudentData struct {
	Faculty    string `json:"faculty"`
	Discipline string `json:"discipline"`
	Name       string `json:"name"` // Opcional: depende de tus datos de Kafka
}

func main() {
	// Configuración del cliente de Redis
	redisClient = redis.NewClient(&redis.Options{
		Addr:     "redis-master:6379", // Nombre del servicio de Redis en Kubernetes
		Password: "",                  // Sin contraseña por defecto
		DB:       0,                   // Utilizar la base de datos 0
	})

	// Configuración de Kafka
	config := sarama.NewConfig()
	config.Consumer.Return.Errors = true

	// Crear un nuevo consumidor de Kafka
	consumer, err := sarama.NewConsumer(kafkaBrokers, config)
	if err != nil {
		log.Fatalf("Error creating Kafka consumer: %v", err)
	}
	defer consumer.Close()

	// Consumir mensajes del topic "winners"
	partitions, _ := consumer.Partitions(topic)
	for _, partition := range partitions {
		pc, _ := consumer.ConsumePartition(topic, partition, sarama.OffsetNewest)
		defer pc.Close()

		for msg := range pc.Messages() {
			fmt.Printf("Received message: %s\n", string(msg.Value))

			// Guardar el mensaje en Redis
			err := processMessage(string(msg.Value), topic)
			if err != nil {
				log.Printf("Error storing message in Redis: %v", err)
			} else {
				log.Printf("Message stored in Redis successfully")
			}
		}
	}
}

// storeInRedis guarda los datos en Redis
func processMessage(data string, topic string) error {
	ctx := context.Background()

	// Deserializar el mensaje JSON a un struct
	var student StudentData
	err := json.Unmarshal([]byte(data), &student)
	if err != nil {
		return fmt.Errorf("error unmarshalling JSON: %v", err)
	}

	// Normalizamos el topic a un nombre sin plural (winners -> winner)
	// topicPrefix := strings.TrimSuffix(topic, "s")

	// Almacenar el conteo por facultad
	facultyKey := fmt.Sprintf("faculty_count:%s", student.Faculty)
	err = redisClient.Incr(ctx, facultyKey).Err()
	if err != nil {
		return fmt.Errorf("error incrementing faculty count: %v", err)
	}

	// Almacenar el conteo por disciplina (solo para ganadores)
	if topic == "winners" {
		disciplineKey := fmt.Sprintf("winner_discipline_count:%s", student.Discipline)
		err = redisClient.Incr(ctx, disciplineKey).Err()
		if err != nil {
			return fmt.Errorf("error incrementing discipline count: %v", err)
		}
	}

	// Almacenar el conteo de perdedores por facultad
	if topic == "losers" {
		loserFacultyKey := fmt.Sprintf("loser_faculty_count:%s", student.Faculty)
		err = redisClient.Incr(ctx, loserFacultyKey).Err()
		if err != nil {
			return fmt.Errorf("error incrementing loser faculty count: %v", err)
		}
	}

	return nil
}
