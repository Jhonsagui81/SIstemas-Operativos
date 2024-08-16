#!/bin/bash

generar_nombre_aleatorio(){
    NUMERO_ALEATORIO=$((RANDOM % 1000))
    NOMBRE="Conteiner_$NUMERO_ALEATORIO"
    echo $NOMBRE
}

generar_nombre_aleatorio

for i in $(seq 1 10); do
    nombres=$(generar_nombre_aleatorio)
    docker run -d --name $nombres alpine
    echo "Contenedor creado: $NOMBRE"
done
