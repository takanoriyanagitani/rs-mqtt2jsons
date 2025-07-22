#!/bin/sh

export MQTT_CLIENT_ID=dummy-client-id-recv
export MQTT_TOPIC=dummy-topic
export MQTT_HOST_IP=127.0.0.1
export MQTT_HOST_PORT=1883

./mqtt2jsons2stdout
