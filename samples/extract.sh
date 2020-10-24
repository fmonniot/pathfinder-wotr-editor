#!/bin/bash

unzip $1 -d extracted/       
#mv extracted/player.json extracted/party.json extracted/header.json ./

python -m json.tool extracted/player.json > player.json
python -m json.tool extracted/party.json > party.json
python -m json.tool extracted/header.json > header.json

rm -r extracted/