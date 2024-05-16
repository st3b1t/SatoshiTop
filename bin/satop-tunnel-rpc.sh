#!/bin/sh

echo "Insert the hostname of Bitcoin node (press Enter for default 'localhost'):"
read host
echo "Insert the SSH username (press Enter for default 'admin'):"
read user
echo "Insert the SSH port (press Enter for default '22'):"
read port

##defaults values
HOST=${host:-localhost}
SSH_USER=${user:-admin}
SSH_PORT=${port:-22}

#
##uncomment to use ssh key file
#ssh -fN -i ~/.ssh/id_rsa -l $SSH_USER -p $SSH_PORT -L 127.0.0.1:8332:localhost:8332 $HOST
#
echo "Creating a secure Bitcoin tunnel listen in 127.0.0.1:8332"
ssh -fN -l $SSH_USER -p $SSH_PORT -L 127.0.0.1:8332:localhost:8332 $HOST
