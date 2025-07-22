#!/bin/sh

port=7218
addr=127.0.0.1
docd=./target

miniserve \
	--port ${port} \
	--interfaces "${addr}" \
	"${docd}"
