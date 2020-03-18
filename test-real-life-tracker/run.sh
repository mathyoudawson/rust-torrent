#! /bin/sh -ea

EXPOSE_TRACKER_ON_PORT=4040

cd $(dirname $0)

echo "[building image]"

docker build -t test-real-life-tracker .

echo "[running image] tracker will be available on localhost:$EXPOSE_TRACKER_ON_PORT"

docker run -p $EXPOSE_TRACKER_ON_PORT:8000 test-real-life-tracker 

