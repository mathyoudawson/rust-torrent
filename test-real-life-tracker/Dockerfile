FROM ubuntu:18.04

RUN apt-get update -y && apt-get install -y npm

RUN useradd -m torrent-tracker

WORKDIR /home/torrent-tracker

USER torrent-tracker

RUN npm install bittorrent-tracker

CMD /home/torrent-tracker/node_modules/bittorrent-tracker/bin/cmd.js
