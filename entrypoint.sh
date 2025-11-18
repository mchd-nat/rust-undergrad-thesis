#!/bin/sh
# Start geckodriver in the background
geckodriver --port 4444 --log debug &
sleep 1
exec /usr/local/bin/app