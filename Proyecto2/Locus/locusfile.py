from locust import HttpUser, TaskSet, task, between
from faker import Faker
import random

# Inicializa Faker
fake = Faker()

class UserBehavior(TaskSet):
    
    @task
    def send_request(self):
        # Genera datos aleatorios usando Faker
        faculty_options = ["Agronomia", "Ingenieria"]
        discipline_options = {0: "natacion", 1: "boxeo", 2: "atletismo"}
        
        # Crea la estructura JSON con datos simulados
        json_payload = {
            "name": fake.name(),
            "age": fake.random_int(min=18, max=30),  # Rango de edad de 18 a 30
            "faculty": random.choice(faculty_options),
            "discipline": random.choice(list(discipline_options.keys()))
        }
        
        # Define la URL según la facultad
        if json_payload["faculty"] == "Agronomia":
            url = "/grpc-go"
        else:
            url = "/grpc-rust"
        
        # Envía la solicitud POST
        self.client.post(url, json=json_payload)
        print(f"Sent: {json_payload}")

class WebsiteUser(HttpUser):
    tasks = [UserBehavior]
    wait_time = between(1, 5)  # Tiempo de espera entre solicitudes para simular tráfico
