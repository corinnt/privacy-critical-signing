#!/bin/bash

ssh-keygen -Y sign -f $1 -n file $2
base64 -i "$2.sig" -o "$2.sig.base"