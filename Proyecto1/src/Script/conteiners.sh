#!/bin/bash

# Array con los nombres de las imágenes
images=("high1" "image_alto_consumo_2" "image_bajo_consumo_1" "image_bajo_consumo_2")

# Función para generar un nombre aleatorio
generate_random_name() {
  # Generamos 10 caracteres alefanuméricos
  random_chars=$(tr -dc 'a-zA-Z0-9' < /dev/urandom | head -c 10)

  # Concatenamos el prefijo "Contenedor" con los caracteres aleatorios
  echo "Contenedor_$random_chars"
}


for i in {1..5}; do
  # Seleccionar una imagen aleatoria
  image_index=$(( RANDOM % ${#images[@]} ))
  image=${images[$image_index]}

  # Generar un nombre aleatorio
  container_name=$(generate_random_name)

  # Crear el contenedor
  docker run -d --name "$container_name" "$image"
done