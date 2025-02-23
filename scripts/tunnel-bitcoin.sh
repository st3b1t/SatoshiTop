#!/bin/sh

BIND1='127.0.0.1'
#docker testing
#BIND2='172.17.0.1'
# -f to background
#
#bitcoin core hostname or ip
BITCOIND=192.168.1.10

ssh -fN -i ~/.ssh/id_rsa -l debian -p 33033 -L $BIND1:8332:localhost:8332 $BITCOIND